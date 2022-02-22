use core::ptr;
use cortex_m::peripheral::DWT;

use crate::processor;

mod systick;

#[repr(C)]
struct ThreadingState {
    currrent: usize,
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
    currrent: 0,
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
        SysTick();
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
                enable_threads();
                return Err(e);
            }
        }

        critical_section::release(cs);
        Ok(())
    }
}
