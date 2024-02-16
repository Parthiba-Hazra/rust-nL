#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rust_os::{memory::BootInfoFrameAllocator, println};
use core::panic::{AssertUnwindSafe, PanicInfo};
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::PageTable;

entry_point!(kernel_main);

#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use rust_os::memory::translate_addr;
    use x86_64::{ structures::paging::{ Page, Translate}, VirtAddr };

    println!("Hello World{}", "!");
    rust_os::init();

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // uncomment line below to trigger a stack overflow
    // stack_overflow();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page 
    //     0x201008,
    //     // some stack page 
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    // let phys_addr_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let table_14 = unsafe { active_level_4_table(phys_addr_mem_offset) };

    // for (i, entry) in table_14.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         // get the physical address from the entry and convert it
    //         let phys_addr = entry.frame().unwrap().start_address();
    //         let virt = phys_addr.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!(" L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // let pointer = 0x20534a as *mut u8;
    // unsafe { let x = *pointer; }
    // println!("read worked");

    // unsafe { *pointer = 42; }
    // println!("write worked");

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rust_os::hault_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hault_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
