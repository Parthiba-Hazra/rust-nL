#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

static HELLO: &[u8] = b"Hello World!!";

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QuemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_quemu(exit_code: QuemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // // Pointer to the VGA buffer address
    // let mut vga_buffer = 0xb8000 as *mut u16;

    // // Iterate through each character in the HELLO message
    // for &char_byte in HELLO.iter() {
    //     // Convert the character to a VGA-compatible format
    //     let color = 0xb;
    //     //the higher 8 bits represent the background and foreground color, and the lower 8 bits represent the ASCII character code.
    //     let char_and_color = (color << 8) | char_byte as u16;

    //     // Use unsafe block to write the character and color to the VGA buffer
    //     unsafe {
    //         // Write the character and color to the buffer
    //         *vga_buffer = char_and_color;

    //         // Move to the next position in the buffer
    //         vga_buffer = vga_buffer.offset(1);
    //     }
    // }

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 37, 7.777).unwrap();
    println!("Hello World {}", ":)");
    
    #[cfg(test)]
    test_main();

    loop {}

}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_quemu(QuemuExitCode::Success);
}

#[test_case]
fn trival_assertion() {
    serial_println!("trival assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}