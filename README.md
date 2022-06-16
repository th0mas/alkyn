# alkyn
Embedded Rust OS with a focus on Erlang style message passing.

[![crates.io](https://img.shields.io/crates/v/alkyn.svg)](https://crates.io/crates/alkyn)
![docs.rs](https://img.shields.io/docsrs/alkyn)
---------

Currently a prototype OS to asses the feasability of using Rust.
Runs on the RP2040 micro-controller.

## Prerequisites
Alkyn requires:

* Rust Nightly
* A [probe-run](https://github.com/knurling-rs/probe-run) compatible debug probe
* An RP2040 based device

Alkyn's examples also use `flip-link` as the linker, which should be installed as follows:
```
cargo install flip-link
```
Alkyn should work without this in your own projects, but its use is recommened.

## Examples
Examples are in the `/examples` directory and can be run on an RP2040
device by running:

```
cargo run --example threads
```