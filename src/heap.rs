//! A heap allocator for cortex-m-processors.
//! Based off cortex-m-alloc, adapted to use rp2040 spinlocks

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::{self, NonNull};
use linked_list_allocator::Heap;

use crate::sync::{self, Spinlock};

pub struct AlkynHeap {
    heap: RefCell<Heap>,
    lock: sync::Spinlock,
}

impl AlkynHeap {
    /// Crate a new UNINITIALIZED heap allocator
    ///
    /// You must initialize this heap using the
    /// [`init`](struct.CortexMHeap.html#method.init) method before using the allocator.
    pub fn empty() -> AlkynHeap {
        AlkynHeap {
            heap: RefCell::new(Heap::empty()),
            lock: Spinlock::new().unwrap(),
        }
    }

    /// Initializes the heap
    ///
    /// This function must be called BEFORE you run any code that makes use of the
    /// allocator.
    ///
    /// `start_addr` is the address where the heap will be located.
    ///
    /// `size` is the size of the heap in bytes.
    ///
    /// Note that:
    ///
    /// - The heap grows "upwards", towards larger addresses. Thus `end_addr` must
    ///   be larger than `start_addr`
    ///
    /// - The size of the heap is `(end_addr as usize) - (start_addr as usize)`. The
    ///   allocator won't use the byte at `end_addr`.
    ///
    /// # Safety
    ///
    /// Obey these or Bad Stuff will happen.
    ///
    /// - This function must be called exactly ONCE.
    /// - `size > 0`
    pub unsafe fn init(&self, start_addr: usize, size: usize) {
      self.lock.critical_section(|| {
        self.init(start_addr, size)
      })
  }
}
