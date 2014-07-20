# Enter the Rust Chamber

This is a compiler that sandboxes software using only the Rust language.

Please do not use Rust as a language-based sandbox.


## Why do this?

The first principle of Rust is that code that does not say the keyword `unsafe` cannot crash (modulo sunspots),
that safe Rust must be *memory safe*,
a term which includes but isn't limited to:

* No use after free.
* No reading uninitialized memory.
* No writing unallocated memory.
* No data races.

Because Rust is so all about memory safety,
*Rust code that has no unsafe blocks and that has no access to libraries
should be able to accomplish little more disruptive than spin the CPU,
trigger unwinding, or recurse into the end of the stack.*

Chamber creates a controlled environment for fuzzing, attacking, and torturing the compiler and libraries.
It provides a framework for attempting to violate Rust's safety guarantees.


## Building

`cargo build`


## Running

```
target/chamber breakme.rs
```

This will create the `breakme` bin. (If you get an error about not finding std
you may need to pass the `--sysroot` flag).

Chamber comes with a simple 'baseline' chamber, `rcr_baseline`,
which reexports nearly all of the Rust Core Library,
and links to it by default.
To specify a different chamber,
pass its name behind the `--chamber` flag:

```
target/chamber breakme.rs --chamber rcr_custom
```

By default Chamber will look in `.`, `./target`, and `./target/deps`,
to find chambers, as well as the normal rustc search paths.
The search path can be augmented with `-L`.

The stock Rust Standard Library itself is a chamber:

```
target/chamber breakme.rs --chamber std
```

The above is equivalent to the default rustc behavior plus Chamber's blacklist plugin.


## How it works

Chamber is a customized Rust compiler.
It links to rustc directly to augment its behavior.
Compared to stock `rustc` there are two major differences:

1. It injects an arbitrary crate as the standard library, including
   prelude and macros. This is called a 'chamber'.

2. It uses lint passes to blacklist unsafe features, including
   linking to any other crate.

Chambers do not need to be 'freestanding';
they may link to std,
and chambered libraries may be intermixed freely with normal Rust libraries.

Chamber is a simple program and is structured for readability.
It is a good demonstration of embedding rustc, as well as creating rustc plugins,
and incorporating both into Cargo packages.
See [`src/chamber/lib.rs`](src/chamber/lib.rs).


## Blacklisted language features

Some Rust features make it easy to break memory safety.
These are turned off.

* `extern crate`
* `unsafe` blocks
* `#[feature(...)]`


## Chambers

Only one chamber exists right now.

* rcr_baseline. This is a chamber that others can build off of. It
  exposes all of the API's from the core library except for
  `core::any`, which has potential issues with forging type hashes,
  and `core::intrinsics`, which I didn't want to look through
  carefully, but mostly can't be called anyway.


## What Rust does and does not promise

TODO: looping, unwinding, stack overflow, memory leaks, abort, oom


## TODO

* Upstream rustc API changes to avoid code duplication.
* Investigate safety of built-in syntax extensions.
* Fix feature gate pass
* Add conveniences API's for compiling .rs, putting the binary into a
  separate process and detecting the special 'ok' crash conditions
  (stack overflow, double fail).
* Investigate impact of native rt injection.
* Add more chambers.
* Fix the way the lints raise errors.
