// Includes code written by valff in the DroneOS project
// Used under the MIT licence.
mod handler;
use core::{arch::asm, intrinsics::size_of};

pub trait Supervisor: Sized + 'static {
    // `SVC_CALL` exception handler for the supervisor
    unsafe extern "C" fn handler();
}

pub trait SvCall<T: SvService>: Supervisor {

    unsafe fn call(service: &mut T);
}

pub trait SvService: Sized + Send + 'static {
    unsafe extern "C" fn handler(&mut self);
}

#[inline(always)]
pub unsafe fn sv_call<T: SvService, const NUM: u8>(service: &mut T) {
    #[cfg(feature = "std")]
    return unimplemented!();
    #[cfg(not(feature = "std"))]
    unsafe {
        if size_of::<T>() == 0 {
            asm!(
                "svc {}",
                const NUM,
                options(nomem, preserves_flags),
            );
        } else {
            asm!(
                "svc {}",
                const NUM,
                options(nomem, preserves_flags),
            );
        }
    }
}

pub fn raw_svc_call<const NUM: u8>() {
    unsafe {
        asm!("svc {imm}", imm = const NUM);
    }
}