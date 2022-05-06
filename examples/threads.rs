#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alkyn::rt::entry;
use defmt::*;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use alkyn::thread::Core;
use hal::pac;
use rp2040_hal as hal;

use alkyn::thread::msg;

use alkyn::thread;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {

    // Load in peripherals
    let pac = pac::Peripherals::take().unwrap();
    let m_pac = cortex_m::Peripherals::take().unwrap();

    // Let alkyn init them, we don't init from within the Kernel
    // so they can be safely used outside
    alkyn::init(pac);


    // Create the Stacks for our processes.
    // Must be static so we can rely on their location in memory.
    static mut STACK1: [u32; 128] = [0xDEADBEEF; 128];
    static mut STACK2: [u32; 128] = [0xDEADBEEF; 128];
    static mut STACK3: [u32; 128] = [0xDEADBEEF; 128];

    // Create processes
    let _ = thread::create_thread("task1", unsafe { &mut STACK1 }, move || {
        info!("Starting task 1!");
        let mut count: i32 = 0;
        msg::Message::new("hello!").send(1).expect("could not send");
        loop {
            let _ = info!("in task {}, count: {} !!", thread::get_current_thread_idx(), count);
            count += 2;
            thread::sleep(500); // sleep for 50 ticks
        }
    });
    let _ = thread::create_thread("task2", unsafe { &mut STACK2 }, move || {
        info!("Starting task 2!");
        loop {
            let _ = info!("in task {} !!", thread::get_current_thread_idx());
            match msg::check_receive() {
                Some(s) => {
                    let v = s.downcast::<&str>().expect("Could not conv to str");
                    info!("Recvd: {}", *v)
                }
                None => (),
            }
            thread::sleep(100); // sleep for 10 ticks
        }
    });
    let _ = thread::create_thread_with_config(
        "task3",
        unsafe { &mut STACK3 },
        || loop {
            thread::sleep(100);
        },
        1,
        false,
        Core::Core1,
    );

    // Start the OS
    alkyn::start(m_pac, 80_000)
}

// End of file
