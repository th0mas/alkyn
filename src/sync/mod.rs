//! Methods to syncronise across cores safely.

use core::sync::atomic::{Ordering};

use crate::hal;
use crate::{pac, processor};

mod mutex;
use defmt::Format;
pub use mutex::Mutex;

#[derive(Clone, Copy, Debug, Format)]
pub struct LockToken(u8);

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


/// We "only" have 30 spinlocks availiable as some are used by the HAL.
/// Indicates which core owns the lock so that we can call critical_section recursively.
///
/// 0 = no one has the lock, 1 = core0 has the lock, 2 = core1 has the lock
/// 4 = lock allocated
static mut LOCK_OWNERS: [u8; 30] = [LOCK_UNOWNED; 30];
pub struct Spinlock {
    lock: LockToken,
}

// Safety: This should be run within a critical section
unsafe fn claim_unused() -> Option<LockToken> {
    LOCK_OWNERS.iter().position(|&x| x == 0).and_then(|x| {
        LOCK_OWNERS[x] = LOCK_ALLOC;
        Some(LockToken(x as u8))
    })
}

unsafe fn unclaim_lock(lock: LockToken) -> LockToken {
    LOCK_OWNERS[lock.0 as usize] = 0;
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

    pub const fn empty() -> Self {
        Self {
            lock: LockToken(30),
        }
    }

    pub fn deinit(&self) -> LockToken {
        unsafe {
            let _sync_lock = hal::sio::Spinlock::<SYNC_LOCK>::claim();
            let lock_index = unclaim_lock(self.lock);
            hal::sio::Spinlock::<SYNC_LOCK>::release();

            self.release();

            lock_index
        }
    }

    pub fn try_claim(&self) -> Option<&Self> {
        let sio = unsafe { &*pac::SIO::ptr() };
        let lock = sio.spinlock[self.lock.0 as usize].read().bits();

        if lock > 0 {
            Some(self)
        } else {
            None
        }
    }

    pub fn claim(&self) -> &Self {
        defmt::trace!("Claiming lock {}", self.lock.0);
        loop {
            if let Some(result) = self.try_claim() {
                break result;
            }
        }
    }

    pub unsafe fn release(&self) {
        let sio = &*pac::SIO::ptr();
        sio.spinlock[self.lock.0 as usize].write_with_zero(|b| b.bits(1))
    }

    pub fn critical_section<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&LockToken) -> R,
    {
        unsafe { processor::disable_interrupts() };
        // Ensure the compiler doesn't re-order accesses and violate safety here
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        let _ = self.claim();
        let r = f(&LockToken(1));
        unsafe {
            self.release();
            processor::enable_interrupts();
        };
        r
    }
}

impl Drop for Spinlock {
    fn drop(&mut self) {
        defmt::trace!("Releasing lock {}", self.lock.0);
        unsafe { self.release() }
    }
}
