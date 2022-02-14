//! [`defmt`](https://github.com/knurling-rs/defmt) global logger over RTT.
//!
//! NOTE when using this crate it's not possible to use (link to) the `rtt-target` crate
//!
//! To use this crate, link to it by importing it somewhere in your project.
//!
//! ```
//! // src/main.rs or src/bin/my-app.rs
//! use defmt_rtt as _;
//! ```
//!
//! # Blocking/Non-blocking
//!
//! `probe-run` puts RTT into blocking-mode, to avoid losing data.
//!
//! As an effect this implementation may block forever if `probe-run` disconnects on runtime. This
//! is because the RTT buffer will fill up and writing will eventually halt the program execution.
//!
//! `defmt::flush` would also block forever in that case.
mod channel;
use channel::Channel;

mod consts;
use consts::*;

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

#[defmt::global_logger]
struct Logger;

/// Global logger lock.
static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_TOKEN: AtomicU8 = AtomicU8::new(0);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        let token = unsafe { critical_section::acquire() };

        if !TAKEN.load(Ordering::Relaxed) {
            // no need for CAS because interrupts are disabled
            TAKEN.store(true, Ordering::Relaxed);

            INTERRUPTS_TOKEN.store(token, Ordering::Relaxed);

            // safety: accessing the `static mut` is OK because we have disabled interrupts.
            unsafe { ENCODER.start_frame(do_write) }
        } else {
            unsafe { critical_section::release(token) };
        }
    }

    unsafe fn flush() {
        // SAFETY: if we get here, the global logger mutex is currently acquired
        handle().flush();
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.end_frame(do_write);
        TAKEN.store(false, Ordering::Relaxed);
        critical_section::release(INTERRUPTS_TOKEN.load(Ordering::Relaxed));
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
  unsafe { handle().write_all(bytes) }
}

#[repr(C)]
struct Header {
  id: [u8; 16],
  max_up_channels: usize,
  max_down_channels: usize,
  up_channel: Channel,
}

// make sure we only get shared references to the header/channel (avoid UB)
/// # Safety
/// `Channel` API is not re-entrant; this handle should not be held from different execution
/// contexts (e.g. thread-mode, interrupt context)
unsafe fn handle() -> &'static Channel {
  // NOTE the `rtt-target` API is too permissive. It allows writing arbitrary data to any
  // channel (`set_print_channel` + `rprint*`) and that can corrupt defmt log frames.
  // So we declare the RTT control block here and make it impossible to use `rtt-target` together
  // with this crate.
  #[no_mangle]
  static mut _SEGGER_RTT: Header = Header {
      id: *b"SEGGER RTT\0\0\0\0\0\0",
      max_up_channels: 1,
      max_down_channels: 0,
      up_channel: Channel {
          name: NAME as *const _ as *const u8,
          buffer: unsafe { &mut BUFFER as *mut _ as *mut u8 },
          size: BUF_SIZE,
          write: AtomicUsize::new(0),
          read: AtomicUsize::new(0),
          flags: AtomicUsize::new(MODE_NON_BLOCKING_TRIM),
      },
  };

  #[cfg_attr(target_os = "macos", link_section = ".uninit,defmt-rtt.BUFFER")]
  #[cfg_attr(not(target_os = "macos"), link_section = ".uninit.defmt-rtt.BUFFER")]
  static mut BUFFER: [u8; BUF_SIZE] = [0; BUF_SIZE];

  static NAME: &[u8] = b"defmt\0";

  &_SEGGER_RTT.up_channel
}