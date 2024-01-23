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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,  // Set the ASCII character for the current position
                    color_code,            // Set the color code for the current position
                };
                self.column_position += 1;  // Move to the next column position
            }
        }
    }

    // Write a string to the screen
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x70 | b'\n' => self.write_byte(byte),  // Write the byte if it's within printable ASCII range or a newline
                _ => self.write_byte(0xfe),  // Write a placeholder character for non-printable ASCII characters
            }
        }
    }

    // Move to a new line
    fn new_line(&mut self) {
        // ... implementation for moving to a new line (not provided in the code snippet)
    }
}


// Function to print a sample text using the Writer
pub fn print() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    // Example usage of the Writer to print "Hello World!"
    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("World!");
}