// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rustc::back::link;
use rustc::driver::config;
use rustc::driver::session::Session;
use rustc::driver::config::CrateType;
use rustc::driver::driver::{collect_crate_metadata, collect_crate_types};
use rustc::driver::driver::{phase_1_parse_input,
                            phase_3_run_analysis_passes,
                            phase_4_translate_to_llvm,
                            phase_5_run_llvm_passes,
                            phase_6_link_output,
                            phase_save_analysis};
use rustc::driver::driver::Input;
use rustc::driver::driver::{stop_after_phase_1,
                            stop_after_phase_2,
                            stop_after_phase_3,
                            stop_after_phase_5};
use rustc::driver::driver::build_output_filenames;
use rustc::front;
use rustc::plugin;
use rustc::plugin::load::Plugins;
use rustc::plugin::registry::Registry;
use rustc::util::common::time;
use serialize::{json, Encodable};
use std::os;
use std::io;
use syntax;
use syntax::ast;
use syntax::diagnostics;
use syntax::parse::token;

use hack_std_inject = std_inject;

pub fn parse_crate_types_from_list(crate_types_list_list: Vec<String>) -> Result<Vec<CrateType>, String> {

    use rustc::driver::config::{CrateTypeExecutable, CrateTypeDylib,
                                CrateTypeRlib, CrateTypeStaticlib};
    use rustc::driver::config::default_lib_output;

    let mut crate_types: Vec<CrateType> = Vec::new();
    for unparsed_crate_type in crate_types_list_list.iter() {
        for part in unparsed_crate_type.as_slice().split(',') {
            let new_part = match part {
                "lib"       => default_lib_output(),
                "rlib"      => CrateTypeRlib,
                "staticlib" => CrateTypeStaticlib,
                "dylib"     => CrateTypeDylib,
                "bin"       => CrateTypeExecutable,
                _ => {
                    return Err(format!("unknown crate type: `{}`",
                                       part));
                }
            };
            crate_types.push(new_part)
        }
    }

    return Ok(crate_types);
}

pub fn monitor(f: proc():Send) {

    use syntax::diagnostic;
    use syntax::diagnostic::Emitter;
    use std::any::AnyRefExt;
    use std::io;
    use std::task::TaskBuilder;

    // FIXME: This is a hack for newsched since it doesn't support split stacks.
    // rustc needs a lot of stack! When optimizations are disabled, it needs
    // even *more* stack than usual as well.
    #[cfg(rtopt)]
    static STACK_SIZE: uint = 6000000;  // 6MB
    #[cfg(not(rtopt))]
    static STACK_SIZE: uint = 20000000; // 20MB

    let (tx, rx) = channel();
    let w = io::ChanWriter::new(tx);
    let mut r = io::ChanReader::new(rx);

    let mut task = TaskBuilder::new().named("rustc").stderr(box w);

    // FIXME: Hacks on hacks. If the env is trying to override the stack size
    // then *don't* set it explicitly.
    if os::getenv("RUST_MIN_STACK").is_none() {
        task = task.stack_size(STACK_SIZE);
    }

    match task.try(f) {
        Ok(()) => { /* fallthrough */ }
        Err(value) => {
            // Task failed without emitting a fatal diagnostic
            if !value.is::<diagnostic::FatalError>() {
                let mut emitter = diagnostic::EmitterWriter::stderr(diagnostic::Auto, None);

                // a .span_bug or .bug call has already printed what
                // it wants to print.
                if !value.is::<diagnostic::ExplicitBug>() {
                    emitter.emit(
                        None,
                        "unexpected failure",
                        None,
                        diagnostic::Bug);
                }

                static BUG_REPORT_URL: &'static str =
                    "http://doc.rust-lang.org/complement-bugreport.html";

                let xs = [
                    "the compiler hit an unexpected failure path. this is a bug.".to_string(),
                    format!("we would appreciate a bug report: {}",
                            BUG_REPORT_URL),
                    "run with `RUST_BACKTRACE=1` for a backtrace".to_string(),
                ];
                for note in xs.iter() {
                    emitter.emit(None, note.as_slice(), None, diagnostic::Note)
                }

                match r.read_to_string() {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        emitter.emit(None,
                                     format!("failed to read internal \
                                              stderr: {}",
                                             e).as_slice(),
                                     None,
                                     diagnostic::Error)
                    }
                }
            }

            // Fail so the process returns a failure code, but don't pollute the
            // output with some unnecessary failure messages, we've already
            // printed everything that we needed to.
            io::stdio::set_stderr(box io::util::NullWriter);
            fail!();
        }
    }
}

pub fn compile_input(sess: Session,
                     cfg: ast::CrateConfig,
                     input: &Input,
                     outdir: &Option<Path>,
                     output: &Option<Path>,
                     altstd: Option<String>,
                     addl_plugins: Plugins) {
    // We need nested scopes here, because the intermediate results can keep
    // large chunks of memory alive and we want to free them as soon as
    // possible to keep the peak memory usage low
    let (outputs, trans, sess) = {
        let (outputs, expanded_crate, ast_map, id) = {
            let krate = phase_1_parse_input(&sess, cfg, input);
            if stop_after_phase_1(&sess) { return; }
            let outputs = build_output_filenames(input,
                                                 outdir,
                                                 output,
                                                 krate.attrs.as_slice(),
                                                 &sess);
            let id = link::find_crate_name(Some(&sess), krate.attrs.as_slice(),
                                           input);
            let (expanded_crate, ast_map)
                = match phase_2_configure_and_expand(&sess, krate, id.as_slice(),
                                                     altstd, addl_plugins) {
                    None => return,
                    Some(p) => p,
                };

            (outputs, expanded_crate, ast_map, id)
        };
        //write_out_deps(&sess, input, &outputs, id.as_slice());

        if stop_after_phase_2(&sess) { return; }

        let analysis = phase_3_run_analysis_passes(sess, &expanded_crate,
                                                   ast_map, id);
        phase_save_analysis(&analysis.ty_cx.sess, &expanded_crate, &analysis, outdir);
        if stop_after_phase_3(&analysis.ty_cx.sess) { return; }
        let (tcx, trans) = phase_4_translate_to_llvm(expanded_crate, analysis);

        // Discard interned strings as they are no longer required.
        token::get_ident_interner().clear();

        (outputs, trans, tcx.sess)
    };
    phase_5_run_llvm_passes(&sess, &trans, &outputs);
    if stop_after_phase_5(&sess) { return; }
    phase_6_link_output(&sess, &trans, &outputs);
}

pub fn phase_2_configure_and_expand(sess: &Session,
                                    mut krate: ast::Crate,
                                    crate_name: &str,
                                    altstd: Option<String>,
                                    addl_plugins: Plugins
                                    )
                                    -> Option<(ast::Crate, syntax::ast_map::Map)> {
    let time_passes = sess.time_passes();

    *sess.crate_types.borrow_mut() =
        collect_crate_types(sess, krate.attrs.as_slice());
    *sess.crate_metadata.borrow_mut() =
        collect_crate_metadata(sess, krate.attrs.as_slice());

    time(time_passes, "gated feature checking", (), |_|
         front::feature_gate::check_crate(sess, &krate));

    krate = time(time_passes, "crate injection", krate, |krate|
                 hack_std_inject::maybe_inject_crates_ref(sess, krate, altstd.clone()));

    // strip before expansion to allow macros to depend on
    // configuration variables e.g/ in
    //
    //   #[macro_escape] #[cfg(foo)]
    //   mod bar { macro_rules! baz!(() => {{}}) }
    //
    // baz! should not use this definition unless foo is enabled.

    krate = time(time_passes, "configuration 1", krate, |krate|
                 front::config::strip_unconfigured_items(krate));

    let Plugins { mut macros, mut registrars }
        = time(time_passes, "plugin loading", (), |_|
               plugin::load::load_plugins(sess, &krate));

    // FIXME: Do this in load_plugins
    let Plugins { macros: addl_macros, registrars: addl_registrars } = addl_plugins;
    macros.push_all_move(addl_macros);
    registrars.push_all_move(addl_registrars);

    let mut registry = Registry::new(&krate);

    time(time_passes, "plugin registration", (), |_| {
        if sess.features.rustc_diagnostic_macros.get() {
            registry.register_macro("__diagnostic_used",
                diagnostics::plugin::expand_diagnostic_used);
            registry.register_macro("__register_diagnostic",
                diagnostics::plugin::expand_register_diagnostic);
            registry.register_macro("__build_diagnostic_array",
                diagnostics::plugin::expand_build_diagnostic_array);
        }

        for &registrar in registrars.iter() {
            registrar(&mut registry);
        }
    });

    let Registry { syntax_exts, lint_passes, .. } = registry;

    {
        let mut ls = sess.lint_store.borrow_mut();
        for pass in lint_passes.move_iter() {
            ls.register_pass(Some(sess), true, pass);
        }
    }

    // Lint plugins are registered; now we can process command line flags.
    if sess.opts.describe_lints {
        //super::describe_lints(&*sess.lint_store.borrow(), true);
        println!("hack: describe_lints removed");
        return None;
    }
    sess.lint_store.borrow_mut().process_command_line(sess);

    // Abort if there are errors from lint processing or a plugin registrar.
    sess.abort_if_errors();

    krate = time(time_passes, "expansion", (krate, macros, syntax_exts),
        |(krate, macros, syntax_exts)| {
            // Windows dlls do not have rpaths, so they don't know how to find their
            // dependencies. It's up to us to tell the system where to find all the
            // dependent dlls. Note that this uses cfg!(windows) as opposed to
            // targ_cfg because syntax extensions are always loaded for the host
            // compiler, not for the target.
            if cfg!(windows) {
                sess.host_filesearch().add_dylib_search_paths();
            }
            let cfg = syntax::ext::expand::ExpansionConfig {
                deriving_hash_type_parameter: sess.features.default_type_params.get(),
                crate_name: crate_name.to_string(),
            };
            syntax::ext::expand::expand_crate(&sess.parse_sess,
                                              cfg,
                                              macros,
                                              syntax_exts,
                                              krate)
        }
    );

    // JBC: make CFG processing part of expansion to avoid this problem:

    // strip again, in case expansion added anything with a #[cfg].
    krate = time(time_passes, "configuration 2", krate, |krate|
                 front::config::strip_unconfigured_items(krate));

    krate = time(time_passes, "maybe building test harness", krate, |krate|
                 front::test::modify_for_testing(sess, krate));

    krate = time(time_passes, "prelude injection", krate, |krate|
                 hack_std_inject::maybe_inject_prelude(sess, krate));

    let (krate, map) = time(time_passes, "assigning node ids and indexing ast", krate, |krate|
         front::assign_node_ids_and_map::assign_node_ids_and_map(sess, krate));

    if sess.opts.debugging_opts & config::AST_JSON != 0 {
        let mut stdout = io::BufferedWriter::new(io::stdout());
        let mut json = json::PrettyEncoder::new(&mut stdout);
        // unwrapping so IoError isn't ignored
        krate.encode(&mut json).unwrap();
    }

    time(time_passes, "checking that all macro invocations are gone", &krate, |krate|
         syntax::ext::expand::check_for_macros(&sess.parse_sess, krate));

    Some((krate, map))
}
