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

use rustc::driver::config::{CrateType, CrateTypeExecutable, CrateTypeDylib,
                            CrateTypeRlib, CrateTypeStaticlib,
                            default_lib_output};
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
    pub crate_types: Vec<CrateType>,
    pub chamber_name: Option<String>,
    pub search_paths: Vec<Path>
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
    let chamber_name = matches.opt_str("chamber-name");
    let search_paths = matches.opt_strs("L").iter().map(|s| Path::new(s.as_slice())).collect();

    Run(Config {
        crate_types: crate_types,
        chamber_name: chamber_name,
        search_paths: search_paths
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
         optmulti("L", "", "Add a directory to the library search path", "PATH"),
         optmulti("", "crate-type", "Comma separated list of types of crates
                                    for the compiler to emit",
                               "[bin|lib|rlib|dylib|staticlib]"),
         optopt("", "chamber", "Chamber name", "CHAMBER")]
}

fn usage() {
    println!("{}", getopts::usage("Usage: chamber [OPTIONS] INPUT",
                                  optgroups().as_slice()));
}

/// The main compilation function.
/// Drives the customized rustc based on a configuration.
pub fn enchamber(config: Config) -> Result<(), String> {
    Ok(())
}
