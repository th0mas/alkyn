use crate::{multi, processor};
use core::ptr;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use defmt::{debug, panic};

use super::ALKYN_THREADS_GLOBAL;

static mut __ALKYN_SYST_ENABLE: bool = false;

#[exception]
fn SysTick() {
    defmt::trace!("systick - iv call");
    let cs = unsafe { critical_section::acquire() };
    let handler = unsafe { &mut ALKYN_THREADS_GLOBAL };
    if handler.inited {
        let count = SYST::get_current();
        if count > handler.prev_cnt {
            handler.counter = handler.counter + count as u64 + (u32::MAX - handler.prev_cnt) as u64
        } else {
            handler.counter = handler.counter + (handler.prev_cnt - count) as u64;
        }
        handler.prev_cnt = count;
    }
    defmt::trace!("systick - Running tick");
    super::run_tick();
    unsafe {
        multi::send_pendsv();
        critical_section::release(cs)
    }
    ctxswitch();
}

#[inline]
fn ctxswitch() {
    let cs = unsafe { critical_section::acquire() };
    let curr_core: usize = processor::get_current_core().into();

    // Safety: We're inside our critical section
    let handler = unsafe { &mut ALKYN_THREADS_GLOBAL };
    let core_state = &mut handler.cores[curr_core];
    if handler.inited {

        if core_state.current == core_state.next {

            defmt::trace!("systick - getting next thr idx");
            core_state.idx = super::get_next_thread_idx();
            unsafe {
                core_state.next = core::intrinsics::transmute(&handler.threads[core_state.idx])
            }
        }
        if core_state.current != core_state.next {
            unsafe {
                defmt::trace!("systick - setting pendsv");
                critical_section::release(cs);
                processor::set_pendsv();
            }
        }
    }

    unsafe { critical_section::release(cs) }
}

pub fn enable(syst: &mut SYST, reload: u32) {
    let cs = unsafe { critical_section::acquire() };

    // Safety: within critical section
    unsafe {
        if !__ALKYN_SYST_ENABLE {
            syst.set_clock_source(SystClkSource::Core);
            syst.set_reload(reload);
            syst.clear_current();
            syst.enable_counter();
            syst.enable_interrupt();
            __ALKYN_SYST_ENABLE = true;
        } else {
            panic!("Tried to enable twice")
        }
    }

    unsafe { critical_section::release(cs) }
}

pub fn run_ctxswitch() {
    defmt::trace!("systick - manual call");
    ctxswitch()
}
