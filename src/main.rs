//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(asm_const)]

use cortex_m_rt::entry;
use defmt::assert;
use defmt::*;
use panic_probe as _;

use cortex_m::asm;
use hal::multicore::{Multicore, Stack};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use hal::pac;
use hal::sio::Sio;
use rp2040_hal as hal;

mod logger;
mod processor;
mod timer;
mod sync;
mod supervisor;
mod thread;
// use sparkfun_pro_micro_rp2040 as bsp;

const CORE1_TASK_COMPLETE: u32 = 0xEE;
static mut CORE1_STACK: Stack<4096> = Stack::new();

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

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    info!("Booting Alkyn");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut m_pac = cortex_m::Peripherals::take().unwrap();
    
    unsafe {
        TIMER = Some(hal::Timer::new(pac.TIMER, &mut pac.RESETS));
    }
    info!("Initializing Core 0");
    let mut sio = Sio::new(pac.SIO);

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio);
    info!("Starting Core 1");

    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(core1_task, unsafe { &mut CORE1_STACK.mem });
    assert!(sio.fifo.read_blocking() == 0, "Core 1 failed");

    info!("Booted");
    info!("Initing threads");
    thread::init(&mut m_pac.SYST, 8000);
}

fn core1_task() -> ! {
    info!("Core 1 Starting");
    let pac = unsafe { pac::Peripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    sio.fifo.write_blocking(0);
    info!("Core 1 started");
    loop {
        info!("Core 1 waiting for work");
        processor::wait_for_interrrupt()
    }
}

// End of file