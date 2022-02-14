// Abstract out some unsafe assembly 
use cortex_m::asm;

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
