# alkyn
Embedded Rust OS with a focus on Erlang style message passing.

---------

Currently a prototype OS to asses the feasability of using Rust.
Runs on the RP2040 micro-controller.

## Prerequisites
Alkyn requires:

* Rust Nightly
* A [probe-run](https://github.com/knurling-rs/probe-run) compatible debug probe
* An RP2040 based device

## Examples
Examples are in the `/examples` directory and can be run on an RP2040
device by running:

```
cargo run --example threads
```