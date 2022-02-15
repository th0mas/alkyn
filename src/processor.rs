// Abstract out some unsafe assembly 
use cortex_m::asm;
use crate::pac;

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

// Get the current core we're executing on.
#[inline]
pub fn get_current_core() -> u8 {
    // Safety: Always safe to read read-only register
    unsafe { (*pac::SIO::ptr()).cpuid.read().bits() as u8 }
}