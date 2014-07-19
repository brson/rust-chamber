# Enter the Rust Chamber

This is a tool for sandboxing code using only the Rust type system.

It is not though an endorsement that Rust is suitable for use as a software sandbox.


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
trigger `fail!`, or hit the end of the stack.*

Chamber creates a controlled environment for fuzzing, attacking, and torturing the compiler.
It provides a framework for attempting to violate Rust's safety guarantees.


# Building

`cargo build`


# Running

```
target/chamber breakme.rs --sysroot=/usr/local
```

This will create the `breakme` bin.

Chamber comes with a simple 'baseline' chamber, `rcr_baseline`,
and links to it by default.
To specify a different chamber,
pass its name behind the `--chamber` flag:

```
target/chamber breakme.rs --chamber rcr_custom
```

By default Chamber will look in `.`, `./target`, and `./target/deps`, in that order,
to find chambers.
The search path can be augmented with `-L`.

# How it works

Chamber is a light wrapper to rustc that injects custom preludes and blacklists language features.
A 'chamber' is just a Rust crate that has the very basic 'shape' of the Rust Standard Library;
primarily, it is a crate with a `prelude` module.

Rust is pretty sweet.

# TODO

* Feature blacklisting.
* Upstream rustc API changes to avoid code duplication.
* Investigate safety of built-in syntax extensions.
