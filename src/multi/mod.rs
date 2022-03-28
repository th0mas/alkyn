use core::sync::atomic;
use defmt::info;
use hal::multicore::{Multicore, Stack};
use hal::{pac, Sio};
use rp2040_hal as hal;
use hal::pac::{interrupt, Interrupt, NVIC};
use core::ptr;

mod alloc;

use crate::processor;

// const NVIC_ICER: u32 = 0xe180;

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
    let _ = core1.spawn(core_boot, unsafe { &mut CORE1_STACK.mem });
    unsafe { CORE1_INIT.store(true, atomic::Ordering::Release) }
    sio.fifo.write_blocking(1);
}

// Boot scheduler on each core
fn core_boot() -> ! {
    info!("Core 1 online");
    unsafe {NVIC::unmask(Interrupt::SIO_IRQ_PROC1);}

    loop {}
}

#[interrupt]
fn SIO_IRQ_PROC1() {
  let pac = unsafe { pac::Peripherals::steal() };
  let mut sio = Sio::new(pac.SIO);

  let msg = sio.fifo.read_blocking();
  match msg {
    1 => defmt::info!("Recvd 1"),
    _ => defmt::error!("Unknown msg")
  }
  
  sio.fifo.write(1);
}