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

use alloc::boxed::Box;

enum ReplyType {
  Reply, NoReply, Stop
}

pub struct Reply<S, M> {
  reply_type: ReplyType,
  state: S,
  message: M
}

/// GenServer implementations
trait GenServer {
    fn handle_call<M, S>(request: Box<dyn Any>, from: usize, state: S) -> Reply<S, M>;
    fn handle_cast<M, S>(request: Box <dyn Any>, from: usize, state: S) -> Reply<S, M>;
    fn handle_info<M, S>(request: Box <dyn Any>, from: usize, state: S) -> Reply<S, M>;
}

struct GenServerHandle<S, T: GenServer> {
  state: S,
  gen_server: T
}