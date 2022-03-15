use core::arch::asm;
use cortex_m::register;
use cortex_m_rt::exception;

use crate::{thread, processor};


#[exception]
fn PendSV() {
    unsafe {
        processor::disable_interrupts(); // We enable later on in asm
        let current_thread = thread::get_current_thread_ptr();
        let mut psp = register::psp::read();
        if current_thread != 0 {
            psp = psp - 16;
            asm!(
                "stmia r0!, {{r4-r7}}",
                "mov r4, r8",
                "mov r5, r9",
                "mov r7, r11",
                "subs r0, #32",
                "stmia r0!, {{r4-r7}}",
                "subs r0, #16", // possibly need another ld here
                "str r0, [{cur}, 0x0]",
                cur = in(reg) current_thread,
                in("r0") psp,
            );
        }
        let next = thread::get_next_thread_ptr();
        let os = &mut thread::__ALKYN_THREADS_GLOBAL;
        os.set_next_to_curr();
        asm!(
            "ldr r3, [{nxt}, 0x0]", // next.sp
            "ldmia r3!, {{r4-r7}}", // Load stack
            "mov r8, r4", // Move to higher vars
            "mov r9,  r5",
            "mov r10, r6",
            "mov r11, r7",
            "ldmia	r3!, {{r4-r7}}", // Load rest of stack
            "msr psp, r3", // Set stack pointer
            "ldr r0, =0xFFFFFFFD", // set to 0
            "cpsie i", // Enable interrupts here
            "bx r0",
            nxt = in(reg) next,
            options(noreturn)
        );
    }
}