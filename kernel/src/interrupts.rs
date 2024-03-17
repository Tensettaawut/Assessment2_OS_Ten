use core::fmt::Write;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::serial;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use crate::HandlerTable;
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1, KeyCode};

// Global storage for handler functions, allowing for dynamic assignment.
lazy_static! {
    static ref HANDLERS: Mutex<Option<HandlerTable>> = Mutex::new(None);
}

// Initialize the Interrupt Descriptor Table (IDT) with specific handlers for different interrupts.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Set handler functions for various types of interrupts.
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// Initializes the interrupt descriptor table (IDT) and assigns handlers.
pub fn init_idt(handlers: HandlerTable) {
    *(HANDLERS.lock()) = Some(handlers);
    IDT.load();
}

/// Handles breakpoint exceptions, primarily used for debugging.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    writeln!(serial(), "EXCEPTION: BREAKPOINT\n{:#?}", stack_frame).unwrap();
}

/// Handles double fault exceptions, which are critical errors that can lead to system crashes.
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// Defines offsets for the Programmable Interrupt Controller (PIC), ensuring no conflicts with CPU exceptions.
const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Initializes the chained PICs to handle hardware interrupts.
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Enum to represent different interrupt indexes, particularly for the timer and keyboard.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    // Helper methods to convert enum variants to their numeric representations.
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// Handles timer interrupts, typically used for scheduling and maintaining system time.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let h = &*HANDLERS.lock();
    if let Some(handler) = h {
        handler.handle_timer();
    }
    unsafe {
        // Notifies the PIC that the interrupt has been handled, allowing for new interrupts.
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Handles keyboard interrupts, translating scancodes to actions within the game.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::Ignore,
        ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = x86_64::instructions::port::Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let h = &*HANDLERS.lock();
            if let Some(handler) = h {
                match key {
                    KeyCode::ArrowLeft | KeyCode::KeyA => handler.move_left(),
                    KeyCode::ArrowRight | KeyCode::KeyD => handler.move_right(),
                    KeyCode::Spacebar => handler.fire_bullet(),
                    _ => {} // Ignore other keys
                }
            }
        }
    }

    unsafe {
        // Notifies the PIC that the interrupt has been handled, allowing for new interrupts.
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
