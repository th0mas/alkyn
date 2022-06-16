extern crate alloc;

use core::any::{Any, TypeId};

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::thread;

// Init needed for static allocation
const INIT: Vec<RawMessage> = Vec::new();

// Mailbox is kept seperate due to lovely Rust memory initialisation hoops
static mut ALKYN_MAILBOX: [Vec<RawMessage>; super::MAX_THREADS] = [INIT; super::MAX_THREADS];

#[derive(Clone, Copy)]
pub struct RawMessage {
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
            _type_id: *&p.type_id(),
            msg: Box::new(p),
        }
    }

    pub fn send(self, idx: usize) -> Result<usize, usize> {
        // Box up our stuff
        let b: Box<dyn Any> = Box::new(*self.msg);
        unsafe {
            let cs = critical_section::acquire();
            ALKYN_MAILBOX[idx].push(RawMessage {
                msg: Box::into_raw(b),
            });

            let handler = &mut super::ALKYN_THREADS_GLOBAL;

            if handler.threads[idx].status == super::ThreadStatus::MailPending
            {
                handler.threads[idx].status = super::ThreadStatus::Ready;

                if handler.threads[idx].priority > handler.threads[thread::get_current_thread_idx()].priority {
                    critical_section::release(cs);
                    thread::systick::run_ctxswitch();
                }
            };
            critical_section::release(cs)
        };
        Ok(idx)
    }
}

pub fn check_receive() -> Option<Box<dyn Any>> {
    let current_thread = super::get_current_thread_idx();
    let msg: Option<RawMessage>;
    unsafe {
        let cs = critical_section::acquire();
        msg = ALKYN_MAILBOX[current_thread].pop();
        critical_section::release(cs)
    };

    match msg {
        Some(m) => {
            let hydrated_msg = unsafe { Box::from_raw(m.msg) };
            Some(hydrated_msg)
        }
        None => None,
    }
}

pub fn receive() -> Box<dyn Any> {
    loop {
        let m = check_receive();
        match m {
            Some(m) => return m,
            None => unsafe {
                let idx = super::get_current_thread_idx();
                super::ALKYN_THREADS_GLOBAL.threads[idx].status =
                    super::ThreadStatus::MailPending;
                thread::sleep(1);
            },
        }
    }
}
