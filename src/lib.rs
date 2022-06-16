//! Alkyn kernel for the RP2040 MCU.
//!
//! The Alkyn kernel is a small prototype Kernel exploring the usability
//! of Erlang-style message passing in microcontroller kernels.
//!
//! It should not, as of June 2022, be considered production ready.
//!
//! # Minimum supported Rust version
//! Nightly, as new as you can get; I've done some terrible things
//!
//! # Safety
//! Horrendous.

#![no_std]
#![feature(core_intrinsics)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(generic_const_exprs)]
#![feature(default_alloc_error_handler)]
#![allow(non_upper_case_globals)]
#![feature(const_btree_new)]
#![feature(ptr_to_from_bits)]

pub use cortex_m_rt as rt;
pub use defmt;

use defmt::info;
use hal::pac;
use panic_probe as _;
use rp2040_hal as hal;

pub mod genserver;
pub mod heap;
pub mod logger;
pub(crate) mod multi;
pub mod processor;
pub(crate) mod supervisor;
pub mod sync;
pub mod thread;

// Setup allocator
use core::mem::MaybeUninit;
use heap::AlkynHeap;
const HEAP_SIZE: usize = 64000; // 64kb

#[global_allocator]
static mut ALLOCATOR: AlkynHeap = AlkynHeap::empty();

static mut TIMER: Option<hal::Timer> = Option::None;

// Setup logging
defmt::timestamp!("{=u8}:{=u32:us}", { processor::get_current_core() }, {
    // safety, this is read only
    unsafe {
        match &TIMER {
            Some(timer) => timer.get_counter_low(),
            None => 0,
        }
    }
});

/// Initialize the kernel.
/// 
/// This MUST be done before using any kernel methods, as they might rely
/// on data structures initualised by this function. 
/// 
/// Wipes Spinlocks and creates a heap.
/// 
/// # Example
/// ```
/// let mut pac = pac::Peripherals::take().unwrap();
/// let mut m_pac = cortex_m::Peripherals::take().unwrap();
/// alkyn::init(pac.TIMER, &mut pac.RESETS);
/// ```
pub fn init(timer: pac::TIMER, resets: &mut pac::RESETS) {
    info!("alkyn: Initing memory and peripherals");
    // Fix spinlocks
    unsafe {
        const SIO_BASE: u32 = 0xd0000000;
        const SPINLOCK0_PTR: *mut u32 = (SIO_BASE + 0x100) as *mut u32;
        const SPINLOCK_COUNT: usize = 32;
        for i in 0..SPINLOCK_COUNT {
            SPINLOCK0_PTR.wrapping_add(i).write_volatile(1);
        }
    }
    info!("alkyn: Initing memory and peripherals");

    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe {
        ALLOCATOR.init((&mut HEAP).as_ptr() as usize, HEAP_SIZE);
        TIMER = Some(hal::Timer::new(timer, resets));
    }
    info!("alkyn: Heap initialized!");
}

/// Starts the Kernel and associated threads.
/// 
/// Should be called last.
/// # Important
/// DOES NOT RETURN
pub fn start(systick: &mut cortex_m::peripheral::SYST, ticks: u32) -> ! {
    info!("alkyn: Starting");
    thread::init(systick, ticks)
}
