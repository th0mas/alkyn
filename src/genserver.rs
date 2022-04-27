//! # Erlang-like GenServer.
//!
//! A GenServer is a process like any other Alkyn Process.
//! It can be used to keep state, execute code asynchronously and so on.
//! The advantage of using a generic server process (Genserver) implemented
//! using this model is that it will have a standard set of interface functions
//! and include functionality for tracing and error reporting.
//!
//! In general, GenServers should be treated like state machines.
//!
extern crate alloc;
use core::any::Any;

use crate::thread;
use alloc::boxed::Box;
use crate::thread::msg;

pub enum MessageType {
    Call,
    Cast,
    Info,
    Destroy,
}

pub enum ReplyType {
    Reply,
    NoReply,
    Stop,
}

pub struct Message {
    message_type: MessageType,
    msg: Box<dyn Any>,
    ptr: usize,
    from: usize
}

pub struct Reply<S> {
    reply_type: ReplyType,
    state: S,
    message: Box<dyn Any>,
}

/// GenServer implementations
pub trait GenServer {
    fn handle_call<S>(request: Box<dyn Any>, from: usize, state: S) -> Reply<S>;
    fn handle_cast<S>(request: Box<dyn Any>, from: usize, state: S) -> Reply<S>;
    fn handle_info<S>(request: Box<dyn Any>, from: usize, state: S) -> Reply<S>;
    fn get_name() -> &'static str;

    fn start<S: Copy>(self, intial_state: S) -> usize
    where
        Self: Sized,
    {
        let h = GenServerHandle {
            state: intial_state,
            gen_server: self,
            name: Self::get_name(),
            ptr: unsafe { core::mem::transmute(0) },
        };

        let idx = h.generate_handler();

        // Immortalize the handler
        // Very leaky
        let b = Box::new(h);
        let ptr = Box::into_raw(b);
        unsafe {
            (*ptr).ptr = ptr;
        };

        idx
    }
}

struct GenServerHandle<S: Copy, T: GenServer> {
    state: S,
    gen_server: T,
    name: &'static str,
    ptr: *mut GenServerHandle<S, T>,
}

impl<S: Copy, T: GenServer> GenServerHandle<S, T> {
    pub fn generate_handler(&self) -> usize {
        static mut STACK: [u32; 1024] = [0; 1024];
        thread::create_thread_with_config(
            self.name,
            unsafe {&mut STACK},
            Self::handle_msg,
            0x1,
            false,
            thread::Core::None,
        ).expect("Could not create genserver tcb");

        0
    }

    fn handle_msg() -> ! {
        loop {
            let m: Result<Box<Message>, _> = thread::msg::receive().downcast();

            match m {
                Ok(parsed) => {
                    let handle = <*mut Self>::from_bits(parsed.ptr);
                    match parsed.message_type {
                        MessageType::Call => {
                            let r = <T>::handle_call(parsed.msg, parsed.from, unsafe {(*handle).state});
                            Self::after_call(handle, r, parsed.from)
                        },
                        MessageType::Cast => todo!(),
                        MessageType::Info => todo!(),
                        MessageType::Destroy => todo!(),
                    };
                },
                Err(a) => defmt::panic!("alkyn: Genserver received bad msg"),
            }
        }
    }

    fn after_call(handle: *mut Self, reply: Reply<S>, from: usize) {
        match reply.reply_type {
            ReplyType::Reply => {
                msg::Message::new(reply.message).send(from).expect("Failed to reply");
            },
            ReplyType::NoReply => (),
            ReplyType::Stop => unsafe {Self::kill(handle)}
        }
        
    }

    unsafe fn kill(handle: *mut Self) {
        thread::kill_thread(thread::get_current_thread_idx());
        let b = Box::from_raw(handle);
        thread::sleep(1);
    }
}
