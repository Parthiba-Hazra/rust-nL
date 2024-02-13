use x86_64::{ structures::paging::PageTable, VirtAddr, };
use x86_64::PhysAddr;
use x86_64::structures::paging::{ OffsetPageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator };

// Intialize a new OffsetPageTable.
//
// This function is unsafe because the caller must guarantee that the complete
// physical memory is mapped to virtual memory at the passed 
// `physical_memory_offset`. Also, this function must be only called once to 
// avoid alising `&mut` references (which is undefined behaviour).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// This function operates on raw pointers (*mut PageTable) and performs 
// manual memory manipulation. Rust's safety guarantees are bypassed here 
// because we're dealing with low-level memory operations.

// The function assumes that the caller has correctly mapped all 
// physical memory to virtual memory at the provided 
// physical_memory_offset. If this assumption is violated, it can 
// lead to undefined behavior, such as accessing invalid memory locations.

// The function returns a mutable reference (&'static mut PageTable). 
// Rust's borrowing rules dictate that only one mutable reference 
// to a piece of data can exist at a time. If this function is called 
// multiple times concurrently or if the returned reference is aliased, 
// it can lead to data races and undefined behavior.

// If this function is called more than once or if the returned 
// mutable reference is aliased 
// (i.e., another mutable reference to the same data exists), 
// it can lead to data inconsistency and undefined behavior.

// Overall, the function is marked as unsafe because it requires the 
// caller to uphold certain invariants and makes assumptions about 
// the correctness of memory layout, which cannot be statically verified 
// by the Rust compiler.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys_addr = level_4_table_frame.start_address();
    let virt_addr = physical_memory_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();

    &mut *page_table_ptr 
}

// Translate the given virtual address to the mapped physical address, or
// `None` if the address is not mapped.
//
// This function is unsafe cause the caller must guareantee that the complete
// physical memory is mapped to virtual memory at the passed 
// `physical_memory_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

// Private function that is called by `transalate_addr`.
//
// This function is safe to limit the scope of `unsafe` because Rust treats
// the whole body of unsafe functions as an unsafe block. This function must
// only be readable through `unsafe fn` from outside of this module.
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // Read the ative level 4 frame from the CR3 register.
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // Translate the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table reference.
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        // Read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // Calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

// This is a example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page, 
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // This is risky 
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}