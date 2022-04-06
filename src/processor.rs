// Abstract out some unsafe assembly 
use cortex_m::{asm, interrupt};
use crate::pac;
use core::ptr;

const ICSR: u32 = 0xE000ED04;

/// Nice CPU helper functions

/// Hint to the CPU to wait for the next interrupt
#[inline]
pub fn wait_for_interrrupt() {
    asm::wfi();
}

// Hint for the CPU top wait for the next event
#[inline]
pub fn wait_for_event() {
    asm::wfe();
}

#[inline]
pub unsafe fn enable_interrupts() {
    interrupt::enable();
}

#[inline]
pub unsafe fn disable_interrupts() {
    interrupt::disable();
}

// Get the current core we're executing on.
#[inline]
pub fn get_current_core() -> u8 {
    // Safety: Always safe to read read-only register
    unsafe { (*pac::SIO::ptr()).cpuid.read().bits() as u8 }
}

#[inline]
pub unsafe fn set_pendsv() {
    let pend = ptr::read_volatile(ICSR as *const u32);
    ptr::write_volatile(ICSR as *mut u32, pend | 1 << 28);
}