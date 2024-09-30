/// A simple bump allocator that allocates memory by "bumping" a pointer
/// forward on each allocation. Memory is not reclaimed until the entire 
/// allocator is reset or destroyed. This type of allocator is very efficient
/// for use cases where all allocations are made and then released in bulk.

use alloc::alloc::{GlobalAlloc, Layout};
use super::{align_up, Locked};
use core::ptr;

/// The `BumpAllocator` struct contains the necessary information to manage
/// heap memory using the bump allocation strategy.
pub struct BumpAllocator {
    /// The start address of the heap memory region managed by this allocator.
    heap_start: usize,

    /// The end address of the heap memory region managed by this allocator.
    heap_end: usize,

    /// The current pointer to the next available memory block for allocation.
    next: usize,

    /// Keeps track of the number of active allocations.
    /// In a simple bump allocator, deallocations do not immediately free memory,
    /// but this counter can help track how many allocations have been made.
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new, empty `BumpAllocator`.
    /// Initially, the heap start and end addresses are set to `0`, indicating
    /// that the allocator is not yet initialized.
    ///
    /// # Returns
    /// A new `BumpAllocator` with no heap memory allocated.
    ///
    /// # Example
    /// ```
    /// let allocator = BumpAllocator::new();
    /// ```
    pub const fn new() -> self::BumpAllocator {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with a given heap start address and size.
    /// This function is `unsafe` because it assumes that the given memory region
    /// is valid and free to use for allocations.
    ///
    /// # Arguments
    /// * `heap_start` - The starting address of the heap memory region.
    /// * `heap_size` - The size of the heap memory region in bytes.
    ///
    /// After initialization, the allocator will be ready to allocate memory from 
    /// the specified heap region.
    ///
    /// # Safety
    /// This function is `unsafe` because it does not perform any checks to ensure 
    /// the memory at the provided address is valid or free to use.
    ///
    /// # Example
    /// ```
    /// let mut allocator = BumpAllocator::new();
    /// unsafe {
    ///     allocator.init(0x1000, 4096); // Initialize with heap starting at 0x1000, with a size of 4KB.
    /// }
    /// ```
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

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

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
