#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alkyn::rt::entry;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use hal::pac;
use rp2040_hal as hal;
use embedded_hal::digital::v2::OutputPin;


use alkyn::thread;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {

    let mut pac = pac::Peripherals::take().unwrap();
    let mut m_pac = cortex_m::Peripherals::take().unwrap();
    alkyn::init(pac.TIMER, &mut pac.RESETS);
    static mut STACK: [u32; 128] = [0xDEADBEEF; 128];

    let _ = thread::create_thread("task1", unsafe {&mut STACK }, move || {
        let mut pac = unsafe {pac::Peripherals::steal()};
        let sio = hal::Sio::new(pac.SIO);

        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        let mut led_pin = pins.gpio25.into_push_pull_output();
            loop {
                led_pin.set_high().unwrap();
                thread::sleep(10);
                led_pin.set_low().unwrap();
                thread::sleep(10);
            }
    });

    alkyn::start(&mut m_pac.SYST, 80_000)
}

// End of file
