#![feature(const_mut_refs)]

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};

// Define the start and end addresses of the heap.
extern "C" {
    static mut __heap_start: u8;
    static _end: u8; // Provided by the linker script
}

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    /// Creates a new BumpAllocator.
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
        }
    }

    /// Initializes the allocator with the given heap bounds.
    /// This function is unsafe because it must not be called more than once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    /// Allocates a block of memory.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            ptr::null_mut() // Out of memory
        } else {
            self.next = alloc_end;
            alloc_start as *mut u8
        }
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();
        bump.alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Deallocation is not supported in a simple bump allocator.
    }
}

/// Aligns the given address `addr` upwards to the nearest multiple of `align`.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// A simple spinlock to provide safe access to the allocator.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Initializes the global allocator.
pub fn init() {
    let heap_start = unsafe { &mut __heap_start as *mut u8 as usize };
    let heap_size = unsafe { &_end as *const u8 as usize - heap_start };
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
