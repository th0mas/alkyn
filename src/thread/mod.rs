use core::ptr;
use cortex_m::peripheral::DWT;
use rp2040_hal::pac::io_bank0::proc0_inte;

use crate::processor;

mod systick;

use cortex_m_rt::exception::SysTick;

#[repr(C)]
pub struct ThreadingState {
    current: usize,
    next: usize,
    inited: bool,
    idx: usize,
    add_idx: usize,
    threads: [ThreadControlBlock; 32],
    counter: u64,
    prev_cnt: u32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThreadStatus {
    Idle,
    Sleeping,
}

/// A single thread's state
#[repr(C)]
#[derive(Clone, Copy)]
struct ThreadControlBlock {
    // start fields used in assembly, do not reorder them
    /// current stack pointer of this thread
    sp: u32,
    privileged: u32, // make it a word, assembly is easier. FIXME
    // end fields used in assembly
    priority: u8,
    status: ThreadStatus,
    sleep_ticks: u32,
}

#[no_mangle]
static mut __ALKYN_THREADS_GLOBAL_PTR: u32 = 0;
static mut __ALKYN_THREADS_GLOBAL: ThreadingState = ThreadingState {
    current: 0,
    next: 0,
    inited: false,
    idx: 0,
    add_idx: 1,
    threads: [ThreadControlBlock {
        sp: 0,
        status: ThreadStatus::Idle,
        priority: 0,
        privileged: 0,
        sleep_ticks: 0,
    }; 32],
    counter: 0,
    prev_cnt: 0,
};

pub fn get_counter() -> u64 {
    unsafe {
        processor::disable_interrupts();
    }

    // Safety: this is only run  on core 0
    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let counter = handler.counter.clone();
    unsafe {
        processor::enable_interrupts();
    }
    counter
}

// Safety: read_only
pub fn get_current_thread_ptr() -> usize {
    unsafe {
        processor::disable_interrupts()
    }

    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let current_thread = handler.current;

    unsafe {
        processor::enable_interrupts()
    }
    current_thread
}

pub fn get_next_thread_ptr() -> usize {
    unsafe {
        processor::disable_interrupts()
    }

    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let next_thread = handler.next;

    unsafe {
        processor::enable_interrupts()
    }
    next_thread
}

/// Initialize the switcher system
pub fn init() -> ! {
    unsafe {
        let cs = critical_section::acquire();
        let ptr: usize = core::intrinsics::transmute(&__ALKYN_THREADS_GLOBAL);
        __ALKYN_THREADS_GLOBAL_PTR = ptr as u32;
        critical_section::release(cs);
        let mut idle_stack = [0xDEADBEEF; 64];
        match create_tcb(
            &mut idle_stack,
            || loop {
                processor::wait_for_event();
            },
            0xff,
            false,
        ) {
            Ok(tcb) => {
                insert_tcb(0, tcb);
            }
            _ => defmt::error!("Alkyn: Could not create idle thread!"),
        }
        __ALKYN_THREADS_GLOBAL.inited = true;
        systick::run_systick();
        loop {
            processor::wait_for_event();
        }
    }
}

pub fn create_thread(stack: &mut [u32], handler_fn: fn() -> !) -> Result<(), u8> {
    create_thread_with_config(stack, handler_fn, 0x00, false)
}

pub fn create_thread_with_config(
    stack: &mut [u32],
    handler_fn: fn() -> !,
    priority: u8,
    priviliged: bool,
) -> Result<(), u8> {
    unsafe {
        let cs = critical_section::acquire();
        let handler = &mut __ALKYN_THREADS_GLOBAL;

        if handler.add_idx >= handler.threads.len() {
            return Err(1); // Too many threads
        }

        if handler.inited && handler.threads[handler.idx].privileged == 0 {
            return Err(2); // Not enough privileges
        }

        match create_tcb(stack, handler_fn, priority, priviliged) {
            Ok(tcb) => {
                insert_tcb(handler.add_idx, tcb);
                handler.add_idx = handler.add_idx + 1;
            }
            Err(e) => {
                critical_section::release(cs);
                return Err(e);
            }
        }

        critical_section::release(cs);
        Ok(())
    }
}

pub fn sleep(ticks: u32) {
    let handler = unsafe {&mut __ALKYN_THREADS_GLOBAL};
     if handler.idx > 0 {
        handler.threads[handler.idx].status = ThreadStatus::Sleeping;
        handler.threads[handler.idx].sleep_ticks = ticks;
        systick::run_systick();
     }
}
