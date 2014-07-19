// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_name = "chamber"]
#![crate_type = "bin"]
#![crate_type = "rlib"]

#![feature(globs)]

extern crate getopts;
extern crate rustc;
extern crate syntax;

use syntax::diagnostics::registry::Registry;
use rustc::back::link::OutputTypeExe;
use rustc::driver::driver::{compile_input, FileInput};
use rustc::driver::config::{CrateType, CrateTypeExecutable, CrateTypeDylib,
                            CrateTypeRlib, CrateTypeStaticlib,
                            default_lib_output, build_configuration};
use SessOpts = rustc::driver::config::Options;
use std::os;
use getopts::{OptGroup, Matches};

pub fn main() {
    let args = os::args();
    let exit_code = match parse_config(args) {
        Run(config) => {
            match enchamber(config) {
                Ok(_) => 0,
                Err(_) => 1
            }
        }
        Help => {
            usage();
            0
        }
        ParseError(s) => {
            println!("error: {}", s);
            1
        }
    };

    os::set_exit_status(exit_code);
}

pub struct Config {
    pub input_file: Path,
    pub crate_types: Vec<CrateType>,
    pub chamber_name: Option<String>,
    pub search_paths: Vec<Path>,
    pub out_dir: Option<Path>,
    pub out_file: Option<Path>,
    pub sysroot: Option<Path>
}

enum ExeMode {
    Run(Config),
    Help,
    ParseError(String)
}

fn parse_config(mut args: Vec<String>) -> ExeMode {

    use getopts::*;

    let _ = args.shift().unwrap();

    if args.is_empty() { return Help }

    let matches = match getopts(args.as_slice(), optgroups().as_slice()) {
        Ok(m) => m,
        Err(f) => return ParseError(f.to_string())
    };

    if matches.opt_present("h") || matches.opt_present("help") { return Help }

    let crate_types = match crate_types_from_matches(&matches) {
        Ok(c) => c,
        Err(s) => return ParseError(s)
    };
    let chamber_name = matches.opt_str("chamber");
    let search_paths = matches.opt_strs("L").iter().map(|s| Path::new(s.as_slice())).collect();

    let out_dir = matches.opt_str("out-dir").map(|o| Path::new(o));
    let out_file = matches.opt_str("o").map(|o| Path::new(o));

    let sysroot = matches.opt_str("sysroot").map(|o| Path::new(o));

    let input_file = match matches.free.len() {
        0 => return Help,
        1 => Path::new(matches.free[0].as_slice()),
        _ => return Help,
    };

    Run(Config {
        input_file: input_file,
        crate_types: crate_types,
        chamber_name: chamber_name,
        search_paths: search_paths,
        out_dir: out_dir,
        out_file: out_file,
        sysroot: sysroot
    })
}

fn crate_types_from_matches(matches: &Matches) -> Result<Vec<CrateType>, String> {
    let mut crate_types: Vec<CrateType> = Vec::new();
    let unparsed_crate_types = matches.opt_strs("crate-type");
    for unparsed_crate_type in unparsed_crate_types.iter() {
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

fn optgroups() -> Vec<OptGroup> {

    use getopts::*;

    vec![optflag("h", "help", "Display this message"),
         optopt("", "chamber",
                "The name of the crate link to as `std`",
                "CHAMBER"),
         optmulti("L", "", "Add a directory to the library search path", "PATH"),
         optmulti("", "crate-type", "Comma separated list of types of crates
                                    for the compiler to emit",
                               "[bin|lib|rlib|dylib|staticlib]"),
         optopt("o", "", "Write output to <filename>", "FILENAME"),
         optopt( "",  "out-dir", "Write output to compiler-chosen filename in <dir>", "DIR"),
         optopt("", "sysroot", "Override the system root", "PATH"),
         ]
}

fn usage() {
    println!("{}", getopts::usage("Usage: chamber [OPTIONS] INPUT",
                                  optgroups().as_slice()));
}

fn build_session_options(config: &Config) -> SessOpts {

    use rustc::driver::config::basic_options;
    use std::cell::RefCell;

    // Convert from Vec<T> to HashSet<T>
    let search_paths = config.search_paths.clone().move_iter().collect();

    SessOpts {
        crate_types: config.crate_types.clone(),
        addl_lib_search_paths: RefCell::new(search_paths),
        maybe_sysroot: config.sysroot.clone(),
        output_types: vec!(OutputTypeExe),
        .. basic_options()
    }
}

/// The main compilation function.
/// Drives the customized rustc based on a configuration.
pub fn enchamber(config: Config) -> Result<(), ()> {

    monitor_for_real(proc() {
        use rustc::driver::session::build_session;

        let ref config = config;

        let sopts = build_session_options(config);
        let source = config.input_file.clone();
        let registry = Registry::new(rustc::DIAGNOSTICS);
        let sess = build_session(sopts, Some(source), registry);
        let cfg = build_configuration(&sess);

        let ref input_file = FileInput(config.input_file.clone());
        let ref out_dir = config.out_dir;
        let ref out_file = config.out_file;

        compile_input(sess, cfg, input_file, out_dir, out_file);
    })
}

fn monitor_for_real(f: proc():Send) -> Result<(), ()> {
    use std::task;

    let res = task::try(proc() {
        monitor(f)
    });

    if res.is_ok() { Ok(()) } else { Err(()) }
}

// Copied from rustc, ugh
fn monitor(f: proc():Send) {

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
