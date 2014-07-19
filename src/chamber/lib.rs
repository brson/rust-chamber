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

extern crate chamber_plugin;
extern crate getopts;
extern crate rustc;
extern crate serialize;
extern crate syntax;

use rustc::driver::config::CrateType;
use rustc::plugin::load::Plugins;
use SessOpts = rustc::driver::config::Options;

pub use driver::main;

mod driver;
mod hacks;
mod std_inject; // also a hack
mod party_favors; // utilities

pub static DEFAULT_CHAMBER: &'static str = "rcr_baseline";

/// Configuration for building Rust source against a chamber.
pub struct Config {
    pub chamber_name: String,
    pub input_file: Path,
    pub crate_types: Vec<CrateType>,
    pub search_paths: Vec<Path>,
    pub out_dir: Option<Path>,
    pub out_file: Option<Path>,
    pub sysroot: Option<Path>
}

/// The main compilation function.
/// Drives the customized rustc based on a configuration.
pub fn enchamber(config: Config) -> Result<(), ()> {

    use hacks::compile_input;
    use rustc::driver::config::build_configuration;
    use rustc::driver::driver::FileInput;
    use syntax::diagnostics::registry::Registry;

    party_favors::monitor_for_real(proc() {
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
