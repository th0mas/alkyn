//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
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
use defmt::info;
use hal::pac;
use panic_probe as _;
use rp2040_hal as hal;

pub mod genserver;
pub mod heap;
pub mod logger;
pub mod multi;
pub mod processor;
pub mod supervisor;
pub mod sync;
pub mod thread;
pub mod timer;

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

pub fn init(pac: &mut pac::Peripherals) {
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
        TIMER = Some(hal::Timer::new(pac.TIMER, &mut pac.RESETS));
    }
    info!("alkyn: Heap initialized!");
}

pub fn start(mut cortex_pac: cortex_m::Peripherals, ticks: u32) -> ! {
    info!("alkyn: Starting");
    thread::init(&mut cortex_pac.SYST, ticks)
}
