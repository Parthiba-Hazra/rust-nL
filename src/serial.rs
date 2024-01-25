use uart_16550::SerialPort; // Import the SerialPort trait from the uart_16550 crate.
use spin::Mutex; // Import the Mutex type from the spin crate.
use lazy_static::lazy_static; // Import the lazy_static macro from the lazy_static crate.

// Define a lazy static global variable named SERIAL1, which is a Mutex wrapping a SerialPort.
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // Create a new SerialPort instance at I/O port 0x3F8.
        let mut serial_port = unsafe {
            SerialPort::new(0x3F8)
        };
        // Initialize the serial port.
        serial_port.init();
        // Return the Mutex wrapping the initialized serial port.
        Mutex::new(serial_port)
    };
}

// Define a hidden function _print that takes a formatting argument and writes it to SERIAL1.
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    // Import the Write trait from core::fmt and write the formatted arguments to SERIAL1.
    use core::fmt::Write;
    // Lock the SERIAL1 Mutex and write the formatted arguments.
    SERIAL1.lock().write_fmt(args).expect("Printing the serial failed");
}

// Define a macro serial_print that prints formatted arguments to the serial port.
#[macro_export]
macro_rules! serial_print {
    // Match any number of formatting arguments and pass them to _print.
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

// Define a macro serial_println that prints formatted arguments to the serial port with a newline.
#[macro_export]
macro_rules! serial_println {
    // Match no arguments and print a newline to the serial port.
    () => ($crate::serial_print!("\n"));
    // Match a single formatting argument and append a newline before printing.
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    // Match multiple formatting arguments and append a newline before printing.
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)* 
    ));
}
