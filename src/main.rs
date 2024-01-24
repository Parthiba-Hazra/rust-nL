#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

static HELLO: &[u8] = b"Hello World!!";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
    loop {}

}
