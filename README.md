# Enter the Rust Chamber

This is a compiler that sandboxes software using only the Rust type system.

Please do not use Rust as a language-based sandbox.

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

`cargo build`.


# Running

```
target/chamber breakme.rs --sysroot=/usr/local
```

This will create the `breakme` bin.

Chamber comes with a simple 'baseline' chamber, `rcr_baseline`,
which reexports nearly all of the Rust Core Library,
and links to it by default.
To specify a different chamber,
pass its name behind the `--chamber` flag:

```
target/chamber breakme.rs --sysroot=/usr/local --chamber rcr_custom
```

By default Chamber will look in `.`, `./target`, and `./target/deps`, in that order,
to find chambers, as well as the normal rustc search paths.
The search path can be augmented with `-L`.

The stock Rust Standard Library itself is a chamber:

```
target/chamber breakme.rs --sysroot=/usr/local --chamber std
```

The above is equivalent to the default rustc behavior plus Chamber's blacklist plugin.

# How it works

Chamber is a Rust compiler.
It works by linking to rustc directly and augmenting its behavior.
It has a few major differences
compared to stock `rustc`:

1. It injects an arbitrary crate as the standard library, including
   prelude and macros. This is called a 'chamber'.

2. It disallows linking to *any other crate*.

3. It disallows `unsafe` blocks.

4. It disallows enabling experimental features ("feature gates").

Chambers do not need to be 'freestanding';
they may link to std,
and chambered libraries may be intermixed freely with normal Rust libraries.

Chamber is a simple program and is structured for readability.
It is a good demonstration of embedding rustc as well as creating rustc plugins.
See [`src/chamber/lib.rs`](src/chamber/lib.rs).

# Blacklisted language features

Some Rust features make it easy to break memory safety.
These are turned off.

* `extern crate`
* `unsafe` blocks
* `#[feature(...)]`

# Chambers

TODO

# What Rust does and does not promise

TODO: looping, unwinding, stack overflow, memory leaks, abort

# TODO

* Upstream rustc API changes to avoid code duplication.
* Investigate safety of built-in syntax extensions.
* Factor out 'default policy' - BASELINE_CHAMBER, search paths
* Fix feature gate pass
* Add conveniences API's for compiling .rs, putting the binary into a
  separate process and detecting the special 'ok' crash conditions
  (stack overflow, double fail).
* Investigate impact of native rt injection.
