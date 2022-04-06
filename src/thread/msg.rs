extern crate alloc;

use core::{
    any::{Any, TypeId},
    fmt::Result,
};

use alloc::boxed::Box;
use alloc::vec::Vec;
use defmt::Format;

use crate::{processor, sync};

// Init needed for static allocation
const INIT: Vec<RawMessage> = Vec::new();

// Mailbox is kept seperate due to lovely rust memory initialisation hoops
static mut ALKYN_MAILBOX: [Vec<RawMessage>; super::MAX_THREADS] = [INIT; super::MAX_THREADS];

#[derive(Clone, Copy)]
pub struct RawMessage {
    idx: usize,
    msg: *mut dyn Any,
}

pub struct Message<T> {
    msg: Box<T>,
    _type_id: TypeId,
}

impl<T> Message<T>
where
    T: 'static,
{
    pub fn new(p: T) -> Message<T> {
        Message {
            msg: Box::new(p),
            _type_id: *&p.type_id(),
        }
    }

    pub fn send(&self, idx: usize) -> Result {
        // Box up our stuff
        let b: Box<dyn Any> = Box::new(self.msg);
        unsafe {
            let cs = critical_section::acquire();
            ALKYN_MAILBOX[idx].push(RawMessage {
                idx: idx,
                msg: Box::into_raw(b),
            })
        };

        Err(())
    }
}
