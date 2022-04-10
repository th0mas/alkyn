#![no_std]
#![no_main]

use alkyn::heap::AlkynHeap;
use alkyn::rt::entry;
use defmt::*;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use alkyn::thread::Core;
use hal::pac;
use rp2040_hal as hal;

use alkyn::thread::msg;

use alkyn::{processor, thread};

// use sparkfun_pro_micro_rp2040 as bsp;

// Setup allocator
use core::mem::MaybeUninit;
const HEAP_SIZE: usize = 64000; // 64kb

#[global_allocator]
static mut ALLOCATOR: AlkynHeap = AlkynHeap::empty();

static mut TIMER: Option<hal::Timer> = Option::None;
static mut stack3: [u32; 256] = [0xDEADBEEF; 256];

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
    // ffs
    unsafe {
        (*pac::SIO::ptr()).spinlock[31].write(|w| w.bits(0));
    }
    info!("Booting Alkyn");
    info!("Alloc'ing heap");
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init((&mut HEAP).as_ptr() as usize, HEAP_SIZE) }

    let mut pac = pac::Peripherals::take().unwrap();
    let mut m_pac = cortex_m::Peripherals::take().unwrap();

    unsafe {
        TIMER = Some(hal::Timer::new(pac.TIMER, &mut pac.RESETS));
    }

    info!("Booted");
    info!("Initing threads");
    static mut stack1: [u32; 128] = [0xDEADBEEF; 128];
    static mut stack2: [u32; 128] = [0xDEADBEEF; 128];
    let _ = thread::create_thread("task1", unsafe { &mut stack1 }, move || {
        info!("Starting task 1!");
        let mut count: i32 = 0;
        msg::Message::new("hello!").send(1).expect("could not send");
        loop {
            let _ = info!("in task 1, count: {} !!", count);
            count += 2;
            thread::sleep(500); // sleep for 50 ticks
        }
    });

    thread::init(&mut m_pac.SYST, 250_000); // 100hz
}

// End of file
