#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alkyn::rt::entry;
use defmt::*;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use hal::pac;
use rp2040_hal as hal;

use alkyn::thread::msg;

use alkyn::thread;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {

    let mut pac = pac::Peripherals::take().unwrap();
    let m_pac = cortex_m::Peripherals::take().unwrap();
    alkyn::init(&mut pac);
    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led_pin = pins.gpio25.into_push_pull_output();

    let _ = thread::create_thread("task1", &mut [0xDEADBEEF; 128], move || {
        loop {
            led_pin.set_high().unwrap();
            thread::sleep(1);
            led_pin.set_low().unwrap();
            thread::sleep(1);
        }
    });

    alkyn::start(m_pac, 80_000)
}

// End of file
