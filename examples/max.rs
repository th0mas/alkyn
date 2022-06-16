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


use alkyn::thread;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {

    // Load in peripherals
    let mut pac = pac::Peripherals::take().unwrap();
    let mut m_pac = cortex_m::Peripherals::take().unwrap();

    // Let alkyn init them, we don't init from within the Kernel
    // so they can be safely used outside
    alkyn::init(pac.TIMER, &mut pac.RESETS);


    // Create the Stacks for our processes.
    // Must be static so we can rely on their location in memory.
    static mut STACKS: [[u32; 128]; 100] = [[0xDEADBEEF; 128]; 100];

    // Create processes

    for stack_index in unsafe {STACKS.iter_mut() }{
        let _ = thread::create_thread("task1", stack_index, move || {
            let mut count: i32 = 0;
            loop {
                let _ = info!("in task {}, count: {} !!", thread::get_current_thread_idx(), count);
                count += 2;
                thread::sleep(100); // sleep for 50 ticks
            }
        });
    }

    // Start the OS
    alkyn::start(&mut m_pac.SYST, 80_000)
}

// End of file
