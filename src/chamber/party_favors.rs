// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// rustc's monitor uses task failure for process error reporting
// (it lets rustc crash). This wraps that behavior with something nicer.
pub fn monitor_for_real(f: proc():Send) -> Result<(), ()> {
    use hacks;
    use std::task;

    let res = task::try(proc() {
        hacks::monitor(f)
    });

    if res.is_ok() { Ok(()) } else { Err(()) }
}

