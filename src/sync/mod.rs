use core::sync::atomic::{AtomicU32, Ordering};

use crate::hal;
use crate::{pac, processor};

const SYNC_LOCK: usize = 30;

// Module for multi-core sync
// These methods should only be used when an item can be accessed
// by *both* cores, otherwise use a Mutex.

/// Marker value to indicate no-one has the lock.
///
/// Initialising `LOCK_OWNER` to 0 means cheaper static initialisation so it's the best choice
const LOCK_UNOWNED: u8 = 0b0000;

// Marker 1 & 2 are used as core markers

/// Marker to show the lock is allocated
const LOCK_ALLOC: u8 = 0b0100;

/// Marker value to indicate that we already owned the lock when we started the `critical_section`.
///
/// Since we can't take the spinlock when we already have it, we need some other way to keep track of `critical_section` ownership.
/// `critical_section` provides a token for communicating between `acquire` and `release` so we use that.
/// If we're the outermost call to `critical_section` we use the values 0 and 1 to indicate we should release the spinlock and set the interrupts back to disabled and enabled, respectively.
/// The value 2 indicates that we aren't the outermost call, and should not release the spinlock or re-enable interrupts in `release`
const LOCK_ALREADY_OWNED: u8 = 0b1000;

/// We "only" have 30 spinlocks availiable as some are used by the HAL.
/// Indicates which core owns the lock so that we can call critical_section recursively.
///
/// 0 = no one has the lock, 1 = core0 has the lock, 2 = core1 has the lock
/// 4 = lock allocated
static mut LOCK_OWNERS: [u8; 30] = [LOCK_UNOWNED; 30];
pub struct Spinlock {
    lock: u8,
}

// Safety: This should be run within a critical section
unsafe fn claim_unused() -> Option<u8> {
    LOCK_OWNERS.iter().position(|&x| x == 0).and_then(|x| {
        LOCK_OWNERS[x] = LOCK_ALLOC;
        Some(x as u8)
    })
}

unsafe fn unclaim_lock(lock: u8) -> u8 {
    LOCK_OWNERS[lock as usize] = 0;
    lock
}

impl Spinlock {
    #[inline]
    pub fn new() -> Option<Self> {
        unsafe {
            let _sync_lock = hal::sio::Spinlock::<SYNC_LOCK>::claim();
            let lock_index = claim_unused();
            hal::sio::Spinlock::<SYNC_LOCK>::release();

            lock_index.and_then(|index| Some(Self { lock: index }))
        }
    }

    fn deinit(&self) -> u8 {
        unsafe {
            let _sync_lock = hal::sio::Spinlock::<SYNC_LOCK>::claim();
            let lock_index = unclaim_lock(self.lock);
            hal::sio::Spinlock::<SYNC_LOCK>::release();

            unsafe { self.release() }

            lock_index
        }
    }

    pub fn try_claim(&self) -> Option<&Self> {
        let sio = unsafe { &*pac::SIO::ptr() };
        let lock = sio.spinlock[self.lock as usize].read().bits();

        if lock > 0 {
            Some(self)
        } else {
            None
        }
    }

    pub fn claim(&self) -> &Self {
        loop {
            if let Some(result) = self.try_claim() {
                break result;
            }
        }
    }

    pub unsafe fn release(&self) {
        let sio = &*pac::SIO::ptr();
        sio.spinlock[self.lock as usize].write_with_zero(|b| b.bits(1))
    }

    pub fn critical_section<F, R>(&self, f:F) -> R
    where F: Fn() -> R {
        unsafe {processor::disable_interrupts() };
        // Ensure the compiler doesn't re-order accesses and violate safety here
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        self.claim();
        let r = f();
        unsafe {self.release(); processor::enable_interrupts();};
        r
    }
}

impl Drop for Spinlock {
    fn drop(&mut self) {
        unsafe { self.release() }
    }
}
