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

    // #[no_mangle] can be used to override weak symbols
    reg.register_lint_pass(box NoManglePass);
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

// NB: Named CH_ because rustc has this same pass.
declare_lint!(CH_UNSAFE_BLOCK, Forbid,
              "`unsafe` blocks")

impl LintPass for UnsafeBlockPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(CH_UNSAFE_BLOCK)
    }

    fn check_expr(&mut self, ctx: &Context, e: &ast::Expr) {
        match e.node {
            // Don't warn about generated blocks, that'll just pollute the output.
            ast::ExprBlock(ref blk) if blk.rules == ast::UnsafeBlock(ast::UserProvided) => {
                ctx.span_lint(CH_UNSAFE_BLOCK, e.span, "chamber: `unsafe` block");
            }
            _ => ()
        }
    }
}

/// Forbids using the `#[feature(...)]` attribute
struct FeatureGatePass;

declare_lint!(CH_FEATURE_GATE, Forbid,
              "enabling experimental features")

impl LintPass for FeatureGatePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(CH_FEATURE_GATE)
    }

    fn check_attribute(&mut self, ctx: &Context, attr: &ast::Attribute) {

        use syntax::attr;

        if attr::contains_name(&[attr.node.value], "feature") {
            ctx.span_lint(CH_FEATURE_GATE, attr.span, "chamber: feature gate");
        }
    }
}

/// Enforces Chamber's restrictions on `extern crate`.
struct CrateLimitPass {
    stdname: String
}

impl CrateLimitPass {
    pub fn new(stdname: String) -> CrateLimitPass {
        CrateLimitPass { stdname: stdname }
    }
}

declare_lint!(CH_CRATE_LIMIT, Forbid,
              "enforces limits on which crates can be linked")

impl LintPass for CrateLimitPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(CH_CRATE_LIMIT)
    }

    fn check_view_item(&mut self, ctx: &Context, item: &ast::ViewItem) {
        match item.node {
            ast::ViewItemExternCrate(std, Some((ref name, _)), _) => {
                // This is the name used in the code.
                if std.as_str() != "std" {
                    ctx.span_lint(CH_CRATE_LIMIT, item.span, "chamber: incorrect ident for std");
                }

                // This is the name of the crate we're calling 'std'.
                if name.get() != self.stdname.as_slice() {
                    ctx.span_lint(CH_CRATE_LIMIT, item.span, "chamber: incorrect name for std");
                }
            }
            ast::ViewItemExternCrate(name, None, _) => {
                if name.as_str() == "native" {
                    // FIXME: This is done by std_inject and exposes chambered
                    // code to the native crate. Bad.
                } else {
                    // std_inject does not emit this pattern
                    ctx.span_lint(CH_CRATE_LIMIT, item.span, "chamber: incorect std `extern crate` form");
                }
            }
            ast::ViewItemUse(_) => ( /* nbd */ )
        }
    }
}

struct NoManglePass;

declare_lint!(CH_NO_MANGLE, Forbid,
              "forbids #[no_mangle]")

impl LintPass for NoManglePass {
    fn get_lints(&self) -> LintArray {
        lint_array!(CH_NO_MANGLE)
    }

    fn check_attribute(&mut self, ctx: &Context, attr: &ast::Attribute) {

        use syntax::attr;

        if attr::contains_name(&[attr.node.value], "no_mangle") {
            ctx.span_lint(CH_NO_MANGLE, attr.span, "chamber: no_mangle");
        }
    }
}

