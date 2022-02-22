use core::arch::asm;
use cortex_m_rt::exception;

#[exception]
unsafe fn SVCall() {
    let mut svc_num: u8 = 1;

    // asm!(
    //     "ldr {0}, [sp, #40]",   // read the PC that was saved before this interrupt happened
    //     "movs {1}, #2",         // store 2 in a reg
    //     "subs {0}, {1}",        // subtract 2 from that PC we recovered
    //     "ldrb {2}, [{0}]",      // read the byte at that position
    //     out (reg) _,
    //     out (reg) _,
    //     lateout (reg) svc_num
    // );

    defmt::info!("svcall #{}", svc_num);
}