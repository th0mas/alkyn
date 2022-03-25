use core::cell::{UnsafeCell};
use super::{LockToken};

pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}
// unsafe impl<T: Send> Send for Mutex<T> {}


impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value)
        }
    }
}


impl <T>Mutex<T> {

  /// Borrows the data for the duration of the spinlock
  pub fn borrow<'sl>(&self, _: &'sl LockToken) -> &'sl T {
    unsafe {&*self.inner.get()}
  }
}

