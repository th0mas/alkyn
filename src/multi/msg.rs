use defmt::Format;

use crate::processor;

#[derive(Format)]
pub struct ThreadControlBlock{
  sp: u32,
  priviledged: u32,
  idx: usize
}

#[derive(Format)]
pub struct ThreadMail<T: Format> {
  idx: usize,
  m: T
}

#[repr(C)]
#[derive(Format)]
pub enum ICCMessage{
  ThreadControlBlock,
  ThreadMail
}

impl ICCMessage {
  pub fn send(self, idx: usize) {
    let core = processor::get_current_core();

  }
}