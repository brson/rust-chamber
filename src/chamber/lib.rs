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

extern crate syntax;
extern crate rustc;

use rustc::driver::config::CrateType;

pub fn main() {
}

struct Config {
    crate_types: Vec<CrateType>,
    chamber_name: Option<String>
}

pub fn parse_config(args: &[&str]) -> Option<Config> {
    extern crate getopts;

    use getopts::*;
}
