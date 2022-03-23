extern crate alloc;

use alloc::boxed::Box;
use defmt::Format;

use crate::processor;

#[derive(Format)]
pub struct ICCMessage<T: Format> {
  idx: usize,
  m: T
}

impl<T: Format> ICCMessage<T> {
  pub fn send(self, idx: usize) {
    let core = processor::get_current_core();
    let msg = Box::new(self);
  }
}

pub fn send<T: Format>(idx: usize, m: &T) {
  
}