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

#![feature(phase, plugin_registrar)]

#[phase(plugin, link)] // Load rustc as a plugin to get lint macros
extern crate rustc;
extern crate syntax;

use rustc::lint::{Context, LintPass, LintArray};
use rustc::plugin::Registry;
use syntax::ast;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_lint_pass(box UnsafeBlockPass);
    // Needs to allow the gates used by std injection
    //reg.register_lint_pass(box FeatureGatePass);
}


declare_lint!(UNSAFE_BLOCK_LINT, Forbid,
              "`unsafe` blocks")

/// Forbids `unsafe` blocks
struct UnsafeBlockPass;

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

declare_lint!(FEATURE_GATE_LINT, Forbid,
              "enabling experimental features")

/// Forbids using the `#[feature(...)]` attribute
struct FeatureGatePass;

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

