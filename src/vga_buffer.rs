// Enum representing different colors for text
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

use volatile::Volatile;
use core::fmt::{Write, Result, Arguments};
use lazy_static::lazy_static;
use spin::Mutex;

// Struct representing the color code for text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    // Constructor to create a new ColorCode
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Struct representing a character on the screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// Constants for the VGA buffer size
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Struct representing the VGA buffer
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Struct representing a text writer for the VGA buffer
pub struct Writer {
    column_position: usize,       // Track the current column position in the VGA buffer
    color_code: ColorCode,        // Store the color information for text
    buffer: &'static mut Buffer,  // Reference to the VGA buffer
}

impl Writer {
    // Write a single byte to the screen
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),  // If the byte is a newline character, move to a new line
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();      // If the current line is full, move to a new line
                }

                let row = BUFFER_HEIGHT - 1;  // Set the row to the last row of the VGA buffer
                let col = self.column_position;  // Get the current column position

                let color_code = self.color_code;  // Get the color code for the text
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,  // Set the ASCII character for the current position
                    color_code,            // Set the color code for the current position
                });
                self.column_position += 1;  // Move to the next column position
            }
        }
    }

    // Write a string to the screen
    // pub fn write_string(&mut self, s: &str) {
    //     for byte in s.bytes() {
    //         match byte {
    //             0x20..=0x70 | b'\n' => self.write_byte(byte),  // Write the byte if it's within printable ASCII range or a newline
    //             _ => self.write_byte(b'*'),  // Use a different placeholder character for non-printable ASCII characters
    //         }
    //     }
    // }

    // Write a string to the screen
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte >= 0x20 && byte <= 0x7E || byte == b'\n' {
                // Printable ASCII character or newline, print the byte
                self.write_byte(byte);
            } else {
                // Non-printable ASCII character, print the placeholder character
                self.write_byte(b'*');
            }
        }
    }


    // Move to a new line in the VGA buffer
    fn new_line(&mut self) {
        // Loop through each row (except the first one)
        for row in 1..BUFFER_HEIGHT {
            // Loop through each column in the buffer
            for col in 0..BUFFER_WIDTH {
                // Read the character from the current row and column
                let character = self.buffer.chars[row][col].read();

                // Write the character to the row above in the same column
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // Clear the last row by filling it with empty characters
        self.clear_row(BUFFER_HEIGHT - 1);

        // Reset the column position to the beginning of the new line
        self.column_position = 0;
    }


    // Clear a specific row in the VGA buffer
    fn clear_row(&mut self, row: usize) {
        // Create a blank character with a space and the current color
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        // Loop through each column in the specified row
        for col in 0..BUFFER_WIDTH {
            // Write the blank character to clear the row
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        self.write_string(s);
        Ok(())
    }
}

// // Function to print a sample text using the Writer
// pub fn print() {
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Yellow, Color::Black),
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };

//     // Example usage of the Writer to print "Hello World!"
//     writer.write_byte(b'H');
//     writer.write_string("ello ");
//     write!(writer, "the numbers are {} and {}", 42, 1.0/3.0).unwrap();
// }

lazy_static!{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {
            &mut *(0xb8000 as *mut Buffer)
        },
    });
}


// Tease are the copy of original macros, just modified to use our own _print function
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::vga_buffer::_print(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Print the given string through the global `WRITER` instance. 
#[doc(hidden)]
pub fn _print(args: Arguments) {
    use core::fmt::write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_prointln_many output");
    }
}

// #[test_case]
// fn test_println_outout() {
//     let s = "Some test string that fits on a single line";
//     println!("{}", s);
//     for (i, c) in s.chars().enumerate() {
//         let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
//         assert_eq!(char::from(screen_char.ascii_character), c);
//     }
// }

#[test_case]
fn test_println_outout() {
    
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}