//! A heap allocator for cortex-m-processors.
//! Based off cortex-m-alloc, adapted to use rp2040 spinlocks

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::{self, NonNull};
use linked_list_allocator::Heap;

use crate::sync::{Mutex, Spinlock};

pub struct AlkynHeap {
    heap: Mutex<RefCell<Heap>>,
    lock: Spinlock,
}

impl AlkynHeap {
    /// Crate a new UNINITIALIZED heap allocator
    ///
    /// You must initialize this heap using the
    /// [`init`](struct.CortexMHeap.html#method.init) method before using the allocator.
    pub const fn empty() -> AlkynHeap {
        AlkynHeap {
            heap: Mutex::new(RefCell::new(Heap::empty())),
            lock: Spinlock::empty(),
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
    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        self.lock = Spinlock::new().unwrap();
        self.lock.critical_section(|t| {
            self.heap.borrow(t).borrow_mut().init(start_addr, size);
        })
    }

    /// Returns an estimate of the amount of bytes in use.
    pub fn used(&self) -> usize {
        self.lock
            .critical_section(|t| self.heap.borrow(t).borrow_mut().used())
    }

    /// Returns an estimate of the amount of bytes available.
    pub fn free(&self) -> usize {
        self.lock
            .critical_section(|t| self.heap.borrow(t).borrow_mut().free())
    }
}

unsafe impl GlobalAlloc for AlkynHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock.critical_section(|t| {
            self.heap
                .borrow(t)
                .borrow_mut()
                .allocate_first_fit(layout)
                .ok()
                .map_or(ptr::null_mut(), |allocation| allocation.as_ptr())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock.critical_section(|t| {
            self.heap
                .borrow(t)
                .borrow_mut()
                .deallocate(NonNull::new_unchecked(ptr), layout)
        });
    }
}
