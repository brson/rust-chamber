# Enter the Rust Chamber

This is a tool for sandboxing software using only the Rust type system.

It is not though a suggestion that Rust is suitable for use as a sandbox.


# Why?

The first principle of Rust is that code that does not say the keyword 'unsafe' cannot crash (modulo sunspots),
that safe Rust must be *memory safe*,
a term which includes but isn't limited to:

* No use after free.
* No reading uninitialized memory.
* No writing unallocated memory.
* No data races.

*A Rust program that cannot use the `unsafe` keyword,
nor link to any libraries,
should be able to accomplish nothing more disruptive than spin the CPU,
trigger unwinding, or recurse into the end of the stack.*

Chamber creates a controlled environment for fuzzing, attacking, and torturing the compiler and libraries.
It provides a framework for attempting to violate Rust's safety guarantees.


# Building

`cargo build`


# Running

```
target/chamber breakme.rs --sysroot=/usr/local
```

This will create the `breakme` bin.

Chamber comes with a simple 'baseline' chamber, `rcr_baseline`,
and links to it by default (warning: `rcr_baseline` currently provides *no features*).
To specify a different chamber,
pass its name behind the `--chamber` flag:

```
target/chamber breakme.rs --sysroot=/usr/local --chamber rcr_custom
```

By default Chamber will look in `.`, `./target`, and `./target/deps`, in that order,
to find chambers, as well as the normal rustc search paths.
The search path can be augmented with `-L`.

The Rust Standard Library itself is a chamber:

```
target/chamber breakme.rs --sysroot=/usr/local --chamber std
```

The above is equivalent to the default rustc behavior plus Chamber's blacklist plugin.

# How it works

Chamber is a light wrapper to rustc that injects custom preludes and blacklists language features, including linking to any other crate.
A 'chamber' is just a Rust crate that has the very basic 'shape' of the Rust Standard Library.
Primarily, it is a crate with a `prelude` module.

Chambers do not need to be 'freestanding';
they may link to std,
and chambered libraries may be intermixed freely with normal Rust libraries.

Rust is pretty sweet.

# Blacklisted language features

Some Rust features make it easy to break memory safety.
These are turned off.

* `unsafe` blocks
* `#[feature(...)]`

# TODO

* Upstream rustc API changes to avoid code duplication.
* Investigate safety of built-in syntax extensions.
* Fill out baseline chamber.
* Add conveniences API's for compiling .rs, putting the binary into a
  separate process and detecting the special 'ok' crash conditions
  (stack overflow, double fail).
* Factor out 'default policy' - BASELINE_CHAMBER, search paths
