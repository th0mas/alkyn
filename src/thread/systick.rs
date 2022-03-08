use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use defmt::panic;
use core::ptr;

use super::__ALKYN_THREADS_GLOBAL;

static mut __ALKYN_SYST_ENABLE: bool = false;

const ICSR: u32 = 0xE000ED04; // 	Interrupt Control and State Register

#[exception]
fn SysTick() {
  systick_handler()
}

#[inline]
fn systick_handler() {
  let cs = unsafe {critical_section::acquire()};

  // Safety: We're inside our critical section
  let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL};

  if handler.inited { // TODO: Do the math here with systick
    let count = SYST::get_current();
    if count > handler.prev_cnt {
      handler.counter = handler.counter + count as u64 + (u32::MAX - handler.prev_cnt) as u64
    } else {
      handler.counter = handler.counter + (count - handler.prev_cnt) as u64;
    }
    handler.prev_cnt = count;
    if handler.current == handler.next {
      // schedule a thread to be run
      handler.idx = get_next_thread_idx();
      unsafe {
        handler.next = core::intrinsics::transmute(&handler.threads[handler.idx])
      }
    }
    if handler.current != handler.next {
      unsafe {
        let pend = ptr::read_volatile(ICSR as *const u32);

        // Set PendSV bit to pending
        ptr::write_volatile(ICSR as *mut u32, pend | 1 << 28);
      }
    }
  }

  unsafe {critical_section::release(cs) }
}

fn enable(syst: &mut SYST, reload: u32) {
  let cs = unsafe {critical_section::acquire()};

  // Safety: within critical section
  unsafe {
    if !__ALKYN_SYST_ENABLE {
      syst.set_reload(reload);
      syst.clear_current();
      syst.enable_counter();
      __ALKYN_SYST_ENABLE = true;
    } else {
      panic!("Tried to enable twice")
    }
    
  }

  unsafe {critical_section::release(cs)}
}

pub fn run_systick() {
  systick_handler()
}