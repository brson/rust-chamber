// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Forbids safety-breaking language features from being used

#![feature(phase)]

#[phase(plugin, link)] // Load rustc as a plugin to get lint macros
extern crate rustc;
extern crate syntax;

use rustc::lint::{Context, LintPass, LintArray};
use rustc::plugin::Registry;
use syntax::ast;

pub fn plugin_registrar(reg: &mut Registry) {

    // Forbid unsafe blocks.
    reg.register_lint_pass(box UnsafeBlockPass);

    // FIXME: Needs to allow the gates used by std injection
    //reg.register_lint_pass(box FeatureGatePass);

    // Temporary hack to get around limitations in plugin API.
    // This plugin needs to know which name we're using for std.
    // It gets it from local_data.
    match get_params() {
        Some(stdname) => {
            // Only allow importing the chamber crate and nothing else
            reg.register_lint_pass(box CrateLimitPass::new(stdname));
        }
        None => {
            // This is probably not intentional.
            fail!("can't get arguments for crate limit");
        }
    }
}

local_data_key!(key_params: String)

pub fn get_params() -> Option<String> {
    key_params.replace(None)
}

pub fn set_params(stdname: String) {
    key_params.replace(Some(stdname));
}


/// Forbids `unsafe` blocks
struct UnsafeBlockPass;

declare_lint!(UNSAFE_BLOCK_LINT, Forbid,
              "`unsafe` blocks")

impl LintPass for UnsafeBlockPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNSAFE_BLOCK_LINT)
    }

    fn check_block(&mut self, ctx: &Context, block: &ast::Block) {

        match block.rules {
            ast::UnsafeBlock(_) => {
                ctx.tcx.sess.span_err(block.span, "chamber: `unsafe` block");
            }
            ast::DefaultBlock => ()
        }
    }
}

/// Forbids using the `#[feature(...)]` attribute
struct FeatureGatePass;

declare_lint!(FEATURE_GATE_LINT, Forbid,
              "enabling experimental features")

impl LintPass for FeatureGatePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(FEATURE_GATE_LINT)
    }

    fn check_attribute(&mut self, ctx: &Context, attr: &ast::Attribute) {

        use syntax::attr;

        if attr::contains_name(&[attr.node.value], "feature") {
            ctx.tcx.sess.span_err(attr.span, "chamber: feature gate");
        }
    }
}

/// Enforces Chamber's restrictions on `extern crate`.
struct CrateLimitPass {
    stdname: String,
    seen: uint
}

impl CrateLimitPass {
    pub fn new(stdname: String) -> CrateLimitPass {
        CrateLimitPass { stdname: stdname, seen: 0 }
    }
}

declare_lint!(CRATE_LIMIT_LINT, Forbid,
              "enforces")

impl LintPass for CrateLimitPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(FEATURE_GATE_LINT)
    }

    fn check_view_item(&mut self, ctx: &Context, item: &ast::ViewItem) {
        match item.node {
            ast::ViewItemExternCrate(std, Some((ref name, _)), _) => {
                // `std` is the value used in code
                if std.as_str() != "std" {
                    ctx.tcx.sess.span_err(item.span, "chamber: incorrect ident for std");
                }

                if name.get() != self.stdname.as_slice() {
                    ctx.tcx.sess.span_err(item.span, "chamber: incorrect name for std");
                }
            }
            ast::ViewItemExternCrate(name, None, _) => {
                if name.as_str() == "native" {
                    // FIXME: This is done by std_inject and exposes chambered
                    // code to the native crate. Bad.
                } else {
                    // std_inject does not emit this pattern
                    ctx.tcx.sess.span_err(item.span, "chamber: incorect std `extern crate` form");
                }
            }
            ast::ViewItemUse(_) => ( /* nbd */ )
        }
    }
}

