// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rust Chamber. Language-based Sandboxing for Rust.
//!
//! Chamber is a Rust compiler. It is implemented by linking to
//! rustc, driving it programmatically. It has two major differences
//! compared to stock `rustc`:
//!
//! 1. It injects an arbitrary crate as the standard library, including
//!    prelude and macros, using rustc's own std_inject pass.
//!
//! 2. It uses lint passes to blacklist unsafe features.

#![feature(globs)]

// The Rust parser.
extern crate syntax;
// The Rust compiler.
extern crate rustc;

// The rustc plugins that implement Chamber's language restrictions.
extern crate chamber_plugin;

extern crate getopts;
extern crate serialize;

use rustc::driver::config::CrateType;
use rustc::plugin::load::Plugins;
use SessOpts = rustc::driver::config::Options;

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
/// Look closely! This is how you drive the Rust compiler
/// the right way.
pub fn enchamber(config: Config) -> Result<(), ()> {

    use hacks::compile_input;
    use rustc::driver::config::build_configuration;
    use rustc::driver::driver::FileInput;
    use rustc::driver::session::build_session;
    use syntax::diagnostics::registry::Registry;

    party_favors::monitor_for_real(proc() {

        let ref config = config;

        let sopts = build_session_options(config);
        let source = config.input_file.clone();
        let registry = Registry::new(rustc::DIAGNOSTICS);
        let sess = build_session(sopts, Some(source), registry);
        let cfg = build_configuration(&sess);

        let ref input_file = FileInput(config.input_file.clone());
        let ref out_dir = config.out_dir;
        let ref out_file = config.out_file;

        let chamber_name = Some(config.chamber_name.clone());

        let plugins = get_chamber_plugins();
        
        compile_input(sess, cfg, input_file, out_dir, out_file, chamber_name, plugins);
    })
}

fn build_session_options(config: &Config) -> SessOpts {

    use rustc::back::link::OutputTypeExe;
    use rustc::driver::config::basic_options;
    use std::cell::RefCell;

    let mut search_paths = config.search_paths.clone();

    // Add some conveniences
    search_paths.push_all([Path::new("."),
                           Path::new("./target"),
                           Path::new("./target/deps")]);

    // Convert from Vec<T> to HashSet<T>
    let search_paths = search_paths.move_iter().collect();

    SessOpts {
        crate_types: config.crate_types.clone(),
        addl_lib_search_paths: RefCell::new(search_paths),
        maybe_sysroot: config.sysroot.clone(),
        output_types: vec!(OutputTypeExe),
        .. basic_options()
    }
}

fn get_chamber_plugins() -> Plugins {
    Plugins {
        macros: vec!(),
        registrars: vec!(chamber_plugin::plugin_registrar)
    }
}
