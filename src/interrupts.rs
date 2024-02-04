use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;


pub enum InterruptIndex {
    Timer = PIC_1_OFFSET as isize,
    Keyboard,
}

// Define offsets for PICs
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

impl From<InterruptIndex> for u8 {
    fn from(in_i: InterruptIndex) -> Self {
        in_i as u8
    }
}

impl From<InterruptIndex> for usize {
    fn from(in_i: InterruptIndex) -> Self {
        in_i as usize
    }
}

// Define a mutex-protected static variable for PICs
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Define the Interrupt Descriptor Table (IDT) as a lazy_static variable
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Set the handler function for the breakpoint exception
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        // Set the handler function and stack index for the double fault exception
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.into()]
            .set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.into()]
            .set_handler_fn(keyboard_interrupt_handler);

        // Return the initialized IDT
        idt
    };
}

// Function to initialize the IDT
pub fn init_idt() {
    // Load the IDT
    IDT.load();
}

// Interrupt handler for the breakpoint exception
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Interrupt handler for the double fault exception
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.into())
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stakc_frame: InterruptStackFrame) {
    
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    
    let scanCode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scanCode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key), 
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.into())
    }
}

// extern "x86-interrupt" fn keyboard_interrupt_handler(_stakc_frame: InterruptStackFrame) {
//     use x86_64::instructions::port::Port;

//     // Create a static array to map scan codes to characters
//     static SCANCODE_MAP: [char; 128] = [
//         '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=',
//         '\0', '\0', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
//         '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\',
//         'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0', '*', '\0', ' ',
//         '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
//         '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
//         '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
//         '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
//         '\0', '\0', '\0', '\0','\0', '\0',
//     ];

//     let mut port = Port::new(0x60);
//     let scan_code: u8 = unsafe { port.read() };

//     // Check if the scan code is within the bounds of the mapping array
//     if (scan_code & 0x80) == 0 {
//         // If it's a regular key press, print the corresponding character
//         let character = SCANCODE_MAP[scan_code as usize];
//         print!("{}", character);
//     }

//     unsafe {
//         PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.into())
//     }
// }

// Test case for the breakpoint exception
#[test_case]
fn test_breakpoint_exception() {
    // Invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
