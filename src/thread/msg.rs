extern crate alloc;

use core::any::Any;

use alloc::vec::Vec;
use defmt::Format;

use crate::processor;

const INIT: Vec<RawMessage> = Vec::new();
static mut ALKYN_MAILBOX: [Vec<RawMessage>; super::MAX_THREADS] = [INIT; super::MAX_THREADS];


#[derive(Clone, Copy)]
pub struct RawMessage {
  idx: usize,
  msg: u8
}

impl RawMessage {
  pub fn send(self, idx: usize) {
    let core = processor::get_current_core();
  }
}

pub fn send<T>(idx: usize, m: &T) {
  let boxed = 8;
  boxed.type_id();
}