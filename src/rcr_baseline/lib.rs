// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_name = "cbr_baseline"]
#![crate_type = "rlib"]

extern crate core;

// This is everything but `any` and `intrinsics`.

pub use core::atomics;
pub use core::bool;
pub use core::cell;
pub use core::clone;
pub use core::cmp;
pub use core::collections;
pub use core::default;
pub use core::failure;
pub use core::finally;
pub use core::f32;
pub use core::f64;
pub use core::fmt;
pub use core::int;
pub use core::iter;
pub use core::i8;
pub use core::i16;
pub use core::i32;
pub use core::i64;
pub use core::kinds;
pub use core::mem;
pub use core::num;
pub use core::ops;
pub use core::option;
pub use core::prelude;
pub use core::ptr;
pub use core::raw;
pub use core::result;
pub use core::simd;
pub use core::slice;
pub use core::str;
pub use core::tuple;
pub use core::ty;
pub use core::uint;
pub use core::u8;
pub use core::u16;
pub use core::u32;
pub use core::unit;
