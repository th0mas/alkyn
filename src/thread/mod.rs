use core::sync::atomic::Ordering;
use core::sync::atomic::compiler_fence;
use cortex_m::{asm, peripheral::SYST};
use defmt::error;

extern crate alloc;
use alloc::vec::{self, Vec};

use crate::processor;
mod msg;

mod systick;

const MAX_THREADS: usize = 32;
const CORES: usize = 2;

#[repr(C)]
pub struct ThreadingState {
    cores: [CoreState; CORES],
    inited: bool,
    add_idx: usize,
    threads: [ThreadControlBlock; MAX_THREADS],
    counter: u64,
    prev_cnt: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CoreState {
    current: usize,
    next: usize,
    idx: usize,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThreadStatus {
    Idle,
    Sleeping,
    MailPending, //
    MailRecv,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Core {
    Core0,
    Core1,
    None,
}

impl Core {
    pub fn from_slice<const N: usize>(pattern: &[u8; N]) -> [Core; N + 1]
    where
        [(); N + 1]: Sized,
    {
        let mut converted = [Core::None; N + 1];
        for (i, c) in pattern.into_iter().enumerate() {
            converted[i] = match c {
                0 => Core::Core0,
                1 => Core::Core1,
                _ => defmt::panic!("thr: Could not find core {}", c),
            }
        }
        converted
    }

    pub fn get_allowed() -> [Core; 2] {
        let current_core = processor::get_current_core();
        Core::from_slice(&[current_core])
    }
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
    core: Core,
    affinity: Core,
}

#[no_mangle]
static mut __ALKYN_THREADS_GLOBAL_PTR: u32 = 0;
pub static mut __ALKYN_THREADS_GLOBAL: ThreadingState = ThreadingState {
    cores: [CoreState {
        current: 0,
        next: 0,
        idx: 0,
    }; CORES],
    inited: false,
    add_idx: CORES,
    threads: [ThreadControlBlock {
        sp: 0,
        status: ThreadStatus::Idle,
        priority: 0,
        privileged: 0,
        sleep_ticks: 0,
        core: Core::None,
        affinity: Core::Core0,
    }; 32],
    counter: 0,
    prev_cnt: 0,
};

impl ThreadingState {
    pub fn set_next_to_curr(&mut self) {
        let core: usize = processor::get_current_core().into();
        self.cores[core].current = self.cores[core].next;
    }
}

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
    unsafe { processor::disable_interrupts() }
    let core: usize = processor::get_current_core().into();

    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let current_thread = handler.cores[core].current;

    unsafe { processor::enable_interrupts() }
    current_thread
}

pub fn get_next_thread_ptr() -> usize {
    unsafe { processor::disable_interrupts() }
    let core: usize = processor::get_current_core().into();
    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let next_thread = handler.cores[core].next;

    unsafe { processor::enable_interrupts() }
    next_thread
}

/// Initialize the switcher system
pub fn init(syst: &mut SYST, ticks: u32) -> ! {
    crate::multi::init_cores();
    unsafe {
        let cs = critical_section::acquire();
        let ptr: usize = core::intrinsics::transmute(&__ALKYN_THREADS_GLOBAL);
        __ALKYN_THREADS_GLOBAL_PTR = ptr as u32;
        create_idle_thr(Core::Core0, 0);
        create_idle_thr(Core::Core1, 1);
        __ALKYN_THREADS_GLOBAL.inited = true;
        critical_section::release(cs);
        systick::enable(syst, ticks);
        systick::run_systick();
        loop {
            processor::wait_for_event();
        }
    }
}


/// Create an idle thread on a core. 
/// 
/// Unsafe as this should only be called once per core, and no guards
/// to make sure you don't do it twice
unsafe fn create_idle_thr(core: Core, idx: usize) {
    let mut idle_stack = [0xDEADBEEF; 64];
        match create_tcb(
            &mut idle_stack,
            || loop {
                processor::wait_for_event();
            },
            0xff,
            false,
            core,
        ) {
            Ok(tcb) => {
                insert_tcb(idx, tcb); // BUG!
            }
            _ => defmt::error!("Alkyn: Could not create idle thread for core!"),
        };
}

pub fn create_thread(stack: &mut [u32], handler_fn: fn() -> !) -> Result<(), u8> {
    create_thread_with_config(stack, handler_fn, 0x00, false, Core::None)
}

pub fn create_thread_with_config(
    stack: &mut [u32],
    handler_fn: fn() -> !,
    priority: u8,
    priviliged: bool,
    affinity: Core,
) -> Result<(), u8> {
    unsafe {
        let cs = critical_section::acquire();
        let handler = &mut __ALKYN_THREADS_GLOBAL;
        let curr_core: usize = processor::get_current_core().into();

        if handler.add_idx >= handler.threads.len() {
            return Err(1); // Too many threads
        }

        if handler.inited && handler.threads[handler.cores[curr_core].idx].privileged == 0 {
            return Err(2); // Not enough privileges
        }

        match create_tcb(stack, handler_fn, priority, priviliged, affinity) {
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
    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    let core_status = handler.cores[processor::get_current_core() as usize];
    if core_status.idx > 0 {
        defmt::debug!("sleep - systick");
        handler.threads[core_status.idx].status = ThreadStatus::Sleeping;
        handler.threads[core_status.idx].sleep_ticks = ticks;
        systick::run_systick();
    }
}

pub fn run_tick() {
    let cs = unsafe { critical_section::acquire() };
    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };
    for i in 1..handler.add_idx {
        if handler.threads[i].status == ThreadStatus::Sleeping {
            if handler.threads[i].sleep_ticks > 0 {
                handler.threads[i].sleep_ticks = handler.threads[i].sleep_ticks - 1;
            } else {
                handler.threads[i].status = ThreadStatus::Idle;
            }
        }
    }

    unsafe { critical_section::release(cs) };
}

pub fn get_next_thread_idx() -> usize {
    // Safety:  Read only

    let handler = unsafe { &mut __ALKYN_THREADS_GLOBAL };

    let new_idx = match handler
        .threads
        .iter()
        .enumerate()
        .filter(|&(_, x)| Core::get_allowed().contains(&x.affinity))
        .filter(|&(idx, x)| (idx > 0) && (idx < handler.add_idx) && (x.status != ThreadStatus::Sleeping))
        .max_by(|&(_, a), &(_, b)| a.priority.cmp(&b.priority))
    {
        Some((idx, _)) => {
            idx
        },
        _ => processor::get_current_core().into(),
    };
    defmt::trace!("thr - nxt idx: {}", new_idx);
    new_idx
}

fn create_tcb(
    stack: &mut [u32],
    handler_fn: fn() -> !,
    priority: u8,
    priviliged: bool,
    affinity: Core,
) -> Result<ThreadControlBlock, u8> {
    if stack.len() < 32 {
        error!("Stack size too small");
        return Err(1);
    }

    let idx = stack.len() - 1;

    let pc: usize = unsafe { core::intrinsics::transmute(handler_fn as *const fn()) };

    // Init registers
    stack[idx] = 1 << 24; // xPSR
    stack[idx - 1] = pc as u32;

    // Fill with dummy vals
    stack[idx - 2] = 0xFFFFFFFD; // return reg
    stack[idx - 3] = 0xCCCCCCCC; // R12
    stack[idx - 4] = 0x33333333; // R3
    stack[idx - 5] = 0x22222222; // R2
    stack[idx - 6] = 0x11111111; // R1
    stack[idx - 7] = 0x00000000; // R0
                                 // aditional regs
    stack[idx - 08] = 0x77777777; // R7
    stack[idx - 09] = 0x66666666; // R6
    stack[idx - 10] = 0x55555555; // R5
    stack[idx - 11] = 0x44444444; // R4
    stack[idx - 12] = 0xBBBBBBBB; // R11
    stack[idx - 13] = 0xAAAAAAAA; // R10
    stack[idx - 14] = 0x99999999; // R9
    stack[idx - 15] = 0x88888888; // R8

    let sp: usize = unsafe { core::intrinsics::transmute(&stack[stack.len() - 16]) };

    let tcb = ThreadControlBlock {
        sp: sp as u32,
        priority: priority,
        privileged: priviliged.into(),
        status: ThreadStatus::Idle,
        sleep_ticks: 0,
        core: Core::None,
        affinity: affinity,
    };
    Ok(tcb)
}

fn insert_tcb(idx: usize, tcb: ThreadControlBlock) {
    defmt::trace!("inserting with idx {}", idx);
    unsafe {
        let handler = &mut __ALKYN_THREADS_GLOBAL;
        handler.threads[idx] = tcb;
    }
}
