#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alkyn::{rt::entry, genserver::GenServer};
use defmt::*;
use panic_probe as _;

use hal::pac;
use rp2040_hal as hal;

use defmt::todo;
extern crate alloc;


use alkyn::genserver;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

struct ExampleGenserver {}
impl GenServer for ExampleGenserver {
    fn handle_call<S>(request: alloc::boxed::Box<dyn core::any::Any>, from: usize, state: S) -> genserver::Reply<S> {
        todo!()
    }

    fn handle_cast<S>(request: alloc::boxed::Box<dyn core::any::Any>, from: usize, state: S) -> genserver::Reply<S> {
        todo!()
    }

    fn handle_info<S>(request: alloc::boxed::Box<dyn core::any::Any>, from: usize, state: S) -> genserver::Reply<S> {
        todo!()
    }

    fn get_name() -> &'static str {
        "Example genserver"
    }
}

#[entry]
fn main() -> ! {

    // Load in peripherals
    let pac = pac::Peripherals::take().unwrap();
    let m_pac = cortex_m::Peripherals::take().unwrap();

    // Let alkyn init them, we don't init from within the Kernel
    // so they can be safely used outside
    alkyn::init(pac);

    let eg = ExampleGenserver{};
    eg.start(1);
    

    // Start the OS
    alkyn::start(m_pac, 80_000)
}

// End of file
