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

#![feature(globs)]

extern crate syntax; // The Rust parser.
extern crate rustc;  // The Rust compiler.

// The rustc plugins that implement Chamber's language restrictions.
extern crate chamber_plugin;

extern crate getopts;
extern crate serialize;

use rustc::driver::config::CrateType;
use rustc::plugin::load::Plugins;
use rustc::driver::config::Options;

// Command line interface.
// Reexported so the source for the `chamber` bin can just call chamber::main().
pub use driver::main;

mod driver;
mod hacks;
mod std_inject; // also a hack
mod party_favors; // utilities

pub static DEFAULT_CHAMBER: &'static str = "rcr_baseline";

/// Configuration for building Rust source against a chamber.
pub struct Config {

    // The name of the 'chamber' (crate) to link to in place of `std`.
    pub chamber_name: String,

    // Normal rustc arguments.

    pub input_file: Path,
    pub crate_types: Vec<CrateType>,
    pub search_paths: Vec<Path>,
    pub out_dir: Option<Path>,
    pub out_file: Option<Path>,
    pub sysroot: Option<Path>
}

/// The main compilation function.
/// Drives the customized rustc based on a configuration.
///
/// Look closely! This is how you drive the Rust compiler properly.
pub fn enchamber(config: Config) -> Result<(), ()> {

    use hacks::compile_input;
    use rustc::driver::config::build_configuration;
    use rustc::driver::driver::FileInput;
    use rustc::driver::session::build_session;
    use syntax::diagnostics::registry::Registry;

    // rustc was designed in another era. It's error handling mechanism
    // involves logging to somewhere and then eventually calling `fail!`.
    //
    // The `monitor_for_real` fn creates a new task with a properly
    // configured environment, calls a closure in which to run the
    // compiler, then monitors for failure.
    //
    // It is a thin wrapper around rustc`s `monitor` function (which does not)
    // intercept the failure. `monitor` does ugly stuff that you don't
    // want to do, like configure the stack size correctly.
    party_favors::monitor_for_real(proc() {

        // This is our own application configuration. We're going to
        // translate it to rustc's (supremely complex) configuration.
        let ref config = config;

        // Build the `Options` struct from our own configuration.
        // `Options` provides the configuration for constructing `Session`,
        // which is the context to run the compiler pipeline one time.
        let sopts = build_session_options(config);

        // Create the "diagnostics registry". This is what
        // maintains error codes and extended error documentation.
        let registry = Registry::new(rustc::DIAGNOSTICS);

        // Create the `Session` from the `Options`.
        // The name of the source file provided here is only to inform
        // debuginfo generation AFAICT.
        let source = config.input_file.clone();
        let sess = build_session(sopts, Some(source), registry);

        // Builds the set of `#[cfg(...)]` idents in effect, combining
        // defaults with those derived from `Session` options.
        let cfg = build_configuration(&sess);

        // This source code comes from a file (`FileInput`),
        // not in memory (`StrInput`).
        let ref input_file = FileInput(config.input_file.clone());

        // Some standard rustc options.
        let ref out_dir = config.out_dir;
        let ref out_file = config.out_file;

        // The name of the library to use for std injection,
        // which we call a 'chamber'.
        let chamber_name = Some(config.chamber_name.clone());

        // Our custom plugins that we want to run.
        let plugins = get_chamber_plugins();
        
        compile_input(sess, cfg, input_file, out_dir, out_file, chamber_name, plugins);
    })
}

/// Converts our app-specific options to a rustc `Options`.
fn build_session_options(config: &Config) -> Options {

    use rustc::back::link::OutputTypeExe;
    use rustc::driver::config::basic_options;
    use std::cell::RefCell;

    let mut search_paths = config.search_paths.clone();

    // Add some conveniences
    search_paths.push_all([Path::new("."),
                           Path::new("./target"),
                           Path::new("./target/deps")]);

    // Convert from Vec<T> to HashSet<T> like magic.
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

        // Don't try to fill out all of `Options` by hand.
        // Use this prototype!
        .. basic_options()
    }
}

fn get_chamber_plugins() -> Plugins {
    Plugins {
        macros: vec!(),
        registrars: vec!(chamber_plugin::plugin_registrar)
    }
}
