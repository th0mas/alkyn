//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(generic_const_exprs)]
#![feature(default_alloc_error_handler)]
#![allow(non_upper_case_globals)]
#![feature(const_btree_new)]

pub use cortex_m_rt as rt;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use hal::pac;
use rp2040_hal as hal;

pub mod heap;
pub mod logger;
pub mod multi;
pub mod processor;
pub mod supervisor;
pub mod sync;
pub mod thread;
pub mod timer;
pub mod genserver;
