use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

// Define the index for the double fault IST (Interrupt Stack Table)
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Define a lazy_static block to initialize the Task State Segment (TSS)
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        // Set the interrupt stack table entry for the double fault IST
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Define the size of the stack for the double fault IST
            const STACK_SIZE: usize = 4096 * 5;
            // Define a static mutable array to represent the stack
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            
            // Get the virtual address of the stack start
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            // Calculate the stack end address
            let stack_end = stack_start + STACK_SIZE;
            // Return the stack end address
            stack_end
        };
        
        // Return the initialized Task State Segment
        tss
    };
}

// Define a lazy_static block to initialize the Global Descriptor Table (GDT) and related selectors
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        // Add a kernel code segment entry to the GDT and get its selector
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        
        // Add a TSS segment entry to the GDT and get its selector
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        
        // Return the initialized GDT and its selectors
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

// Define a structure to hold the GDT selectors
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// Function to initialize the GDT and set CS and TSS registers
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    
    // Set the CS register to the code selector
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        // Load the TSS selector
        load_tss(GDT.1.tss_selector);
    }
}
