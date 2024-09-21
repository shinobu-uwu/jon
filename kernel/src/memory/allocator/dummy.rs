use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::println;

pub struct Dummy {
    heap_start: AtomicUsize,
    heap_end: usize,
}

impl Dummy {
    pub const fn new() -> Self {
        Dummy {
            heap_start: AtomicUsize::new(0),
            heap_end: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start.store(heap_start, Ordering::Relaxed);
        self.heap_end = heap_start + heap_size;
    }
}

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap_start = self.heap_start.load(Ordering::Relaxed);
        let alloc_start = (heap_start + layout.align() - 1) & !(layout.align() - 1);

        if alloc_start + layout.size() > self.heap_end {
            null_mut() // Out of memory
        } else {
            println!("Alloced");
            self.heap_start
                .store(alloc_start + layout.size(), Ordering::Relaxed);
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // For simplicity, we're not implementing dealloc in this simple allocator.
    }
}
