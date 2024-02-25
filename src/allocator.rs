use alloc::alloc::{GlobalAlloc, Layout};
use core::{ptr::null_mut};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    }, 
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

/// A dummy allocator that always returns null pointers for allocation requests
pub struct Dummy;

/// The starting address of the heap
pub const HEAP_START: usize = 0x_7777_7777_7777;

/// The size of the heap in bytes
pub const HEAP_SIZE: usize = 700 * 1024;

/// The global allocator instance
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe impl GlobalAlloc for Dummy {
    /// Allocates memory according to the specified layout.
    /// This function always returns a null pointer.
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    /// Deallocates memory.
    /// This function panics since deallocation should never be called in this context.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should never be called")
    }
}

/// Initializes the heap by mapping physical frames to virtual memory pages.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>, 
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    // Create a range of pages that cover the entire heap
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Map each page of the heap to a physical frame
    for page in page_range {
        let frame = frame_allocator.allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}