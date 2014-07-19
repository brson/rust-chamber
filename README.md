# Enter the Rust Chamber

This is a tool for sandboxing code using only the Rust type system.

It is not though an endorsement that Rust is suitable for use as a software sandbox.


# Why then?

The first principle of Rust is that code that does not say the keyword 'unsafe' cannot crash (modulo sunspots).
More firmly, that safe Rust must be *memory safe*, a term which includes but isn't limited to:

* No use after free.
* No reading uninitialized memory.
* No writing unallocated memory.
* No data races.

A Rust program that cannot use the `unsafe` keyword,
nor link to any libraries,
should be able to accomplish nothing more disruptive than spin the CPU,
trigger `fail!`, or hit the end of the stack.

Chamber creates a controlled environment for fuzzing, attacking, and torturing the compiler.
It provides a framework for attempting to violate Rust's safety guarantees.

Enough of such attempts and maybe Rust *can* be trusted as a sandbox.


# Building

Use Cargo, y'all.


# Running

```
chamber breakme.rs
```

This will create the `breakme` bin.
The `--crate-type` flag and `crate_type` attribute also work.

Chamber comes with a simple 'baseline' chamber and links to it by default.
To specify a different chamber,
pass its path behind the `-c` flag:

```
chamber breakme.rs -c libcustomchamber.rlib
```


# How it works

Chamber is a light wrapper to rustc that injects custom preludes and blacklists language features.
A 'chamber' is just a Rust crate that has the very basic 'shape' of the Rust Standard Library;
primarily, it is a crate with a `prelude` module.

Rust is pretty sweet.
