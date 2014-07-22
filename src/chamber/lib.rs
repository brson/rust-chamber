// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rust Chamber. Language-based sandboxing for Rust.
//!
//! Chamber is a customized Rust compiler. It is implemented by
//! linking to rustc, driving it programmatically (i.e. it doesn't
//! execute the `rustc` binary in another process). It has two major
//! differences compared to stock `rustc`:
//!
//! 1. It injects an arbitrary crate as the standard library, including
//!    prelude and macros, using rustc's own std_inject pass.
//!
//! 2. It uses lint passes to blacklist unsafe features.
//!
//! This file is written to clearly demonstrate how to embed
//! the Rust compiler.


extern crate syntax; // The Rust parser.
extern crate rustc;  // The Rust compiler.

// The rustc plugins that implement Chamber's language restrictions.
extern crate chamber_plugin;

// The stock command line parsing lib.
extern crate getopts;

// Just a few types needed in top level declarations.
use rustc::driver::config::CrateType;
use rustc::plugin::load::Plugins;
use rustc::driver::config::Options;

// The command line interface, including `main`.
pub mod driver;


/// Configuration for building Rust source against a chamber.
pub struct Config {

    // The crate source to 'enchamber' (compile).
    pub input_file: Path,

    // The name of the 'chamber' (crate) to link to in place of `std`.
    pub chamber_name: String,

    // Normal rustc arguments.

    pub crate_types: Vec<CrateType>, // --crate-type
    pub search_paths: Vec<Path>,     // -L
    pub out_dir: Option<Path>,       // --out-dir
    pub out_file: Option<Path>,      // --out-file, -o
    pub sysroot: Option<Path>        // --sysroot
}

/// The main compilation function.
/// Drives the customized rustc based on a configuration.
///
/// Look closely! This is how you drive the Rust compiler properly.
pub fn enchamber(config: Config) -> Result<(), ()> {

    use rustc::driver::config::build_configuration;
    use rustc::driver::driver::{compile_input, FileInput};
    use rustc::driver::session::{Session, build_session};
    use syntax::ast;
    use syntax::diagnostics::registry::Registry;

    // rustc was designed in another era. It's error handling mechanism
    // involves logging to somewhere and then eventually calling `fail!`.
    // Since rustc is also prone to ICEing, you really must put it into
    // it's own task to run it reliably (score 1 for task isolation and
    // recovery).
    //
    // The `monitor_for_real` fn creates a new task with a properly
    // configured environment, calls a closure in which to run the
    // compiler, then monitors for failure.
    //
    // It is a thin wrapper around rustc`s `monitor` function (which
    // does not intercept the failure - rustc essentially crashes on
    // error). `monitor` does ugly stuff that you don't want to do,
    // like configure rustc's surprisingly complex error handling and
    // doing something useful with ICE's.
    monitor_for_real(proc() {

        // This is our own application configuration. We're going to
        // translate it to rustc's (moderately complex) configuration.
        let ref config = config;

        // Build the `Options` struct from our own configuration.
        // `Options` provides the configuration for constructing `Session`,
        // which is the context to run the compiler pipeline one time.
        let sopts: Options = build_session_options(config);

        // Create the "diagnostics registry". This is what
        // maintains error codes and extended error documentation.
        let registry: Registry = Registry::new(rustc::DIAGNOSTICS);

        // Create the `Session` from the `Options`.
        // The name of the source file provided here is only to inform
        // debuginfo generation AFAICT.
        let source = config.input_file.clone();
        let sess: Session = build_session(sopts, Some(source), registry);

        // Builds the set of `#[cfg(...)]` idents in effect, combining
        // defaults with those derived from `Session` options.
        let cfg: ast::CrateConfig = build_configuration(&sess);

        // This source code comes from a file (`FileInput`),
        // not in memory (`StrInput`).
        let ref input_file = FileInput(config.input_file.clone());

        // Some standard rustc options.
        let ref out_dir = config.out_dir;
        let ref out_file = config.out_file;

        // Our custom plugins that we want to run.
        let plugins: Plugins = get_chamber_plugins(config);
        
        compile_input(sess, cfg, input_file, out_dir, out_file, Some(plugins));
    })
}

/// Converts our app-specific options to a rustc `Options`.
fn build_session_options(config: &Config) -> Options {

    use rustc::back::link::OutputTypeExe;
    use rustc::driver::config::basic_options;
    use std::cell::RefCell;

    // Convert from Vec<T> to HashSet<T> like magic.
    let search_paths = config.search_paths.clone();
    let search_paths = search_paths.move_iter().collect();

    Options {
        // If this is empty rustc will just pick a crate type.
        crate_types: config.crate_types.clone(),

        // -L paths.
        addl_lib_search_paths: RefCell::new(search_paths),

        // The "sysroot" a directory rustc uses as a reference point
        // for various operations, including discovering crates. It is
        // often "/usr/local". *By default rustc infers it to be the
        // directory above the directory containing the running
        // executeable.* In our case that executable is probably
        // called `chamber` and is not located anywhere near the
        // sysroot.
        maybe_sysroot: config.sysroot.clone(),

        // Output a final binary. rustc will output nothing by default.
        output_types: vec!(OutputTypeExe),

        // The name of the library we'll be using as 'std', the 'chamber'.
        alt_std_name: Some(config.chamber_name.clone()),

        // Don't try to fill out all of `Options` by hand.
        // Use this prototype!
        .. basic_options()
    }
}

fn get_chamber_plugins(config: &Config) -> Plugins {

    // HACK: Configure the plugins via local_data since
    // there's no way to pass it through the plugin registrar.
    chamber_plugin::set_params(config.chamber_name.clone());

    Plugins {
        macros: vec!(),
        registrars: vec!(chamber_plugin::plugin_registrar)
    }
}

// rustc's monitor uses task failure for process error reporting
// (it lets rustc crash). This wraps that behavior with something nicer.
pub fn monitor_for_real(f: proc():Send) -> Result<(), ()> {
    use rustc::driver::monitor;
    use std::task;

    let res = task::try(proc() {
        monitor(f)
    });

    if res.is_ok() { Ok(()) } else { Err(()) }
}

