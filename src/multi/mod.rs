use core::sync::atomic;
use defmt::info;
use hal::multicore::{Multicore, Stack};
use hal::{pac, Sio};
use rp2040_hal as hal;

mod msg;
mod alloc;

use crate::processor;

static mut CORE1_STACK: Stack<4096> = Stack::new();

static mut CORE1_INIT: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn init_cores() {
    // Initialize message heap
    
    // Safety: We only use the required fields in this mod
    let mut pac = unsafe { pac::Peripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio);

    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _ = core1.spawn(core_loop, unsafe { &mut CORE1_STACK.mem });
    unsafe { CORE1_INIT.store(true, atomic::Ordering::Release) }
}

// Boot scheduler on each core
fn core_loop() -> ! {
    info!("Core 1 online");
    let pac = unsafe { pac::Peripherals::steal() };

    let mut sio = Sio::new(pac.SIO);
    loop {
        processor::wait_for_event();
        let m = sio.fifo.read_blocking();

        // Safety: lmao
        let m: &msg::ICCMessage  = unsafe {core::intrinsics::transmute(m)};
        defmt::info!("{}", m);
    }
}
