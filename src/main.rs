//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use panic_probe as _;

use cortex_m::{asm};
use hal::multicore::{Multicore, Stack};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp2040_hal as hal;
use hal::pac;
use hal::sio::Sio;

mod logger;
// use sparkfun_pro_micro_rp2040 as bsp;

const CORE1_TASK_COMPLETE: u32 = 0xEE;
static mut CORE1_STACK: Stack<4096> = Stack::new();

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;


#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut sio = Sio::new(pac.SIO);

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio);
    let cores = mc.cores();
    let core1 = &mut cores[1];

    let _test = core1.spawn(core1_task, unsafe {&mut CORE1_STACK.mem });
    sio.fifo.write_blocking(1);

    info!("Have replied to Core 1");

    loop {
        asm::nop();
    }
}

fn core1_task() -> ! {
    let pac = unsafe{pac::Peripherals::steal()};
    let mut sio = Sio::new(pac.SIO);
    info!("Core 1 says hello!");
    let reply = sio.fifo.read_blocking();
    info!("Core 0 has replied: {}", reply);
    loop {

    }

}

// End of file
