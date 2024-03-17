// Original code from rust-osdev/bootloader crate https://github.com/rust-osdev/bootloader

#![no_std]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod allocator; // Make sure to include your allocator module.

use core::cell::UnsafeCell;
use core::panic::PanicInfo;
use core::fmt::Write;
use uart_16550::SerialPort;
use pc_keyboard::DecodedKey;
extern crate alloc;

// Function to initialize and get access to the serial port for debugging.
pub fn serial() -> SerialPort {
    let mut port = unsafe { SerialPort::new(0x3F8) };
    port.init();
    port
}

// Struct for handling various types of interrupts and system events.
pub struct HandlerTable {
    timer: Option<fn()>,
    keyboard: Option<fn(DecodedKey)>,
    startup: Option<fn()>,
    cpu_loop: fn() -> !,
}

impl HandlerTable {
    // Constructor for HandlerTable.
    pub fn new() -> Self {
        HandlerTable {timer: None, keyboard: None, startup: None, cpu_loop: hlt_loop}
    }

    // Initializes the system with the configured handlers and starts the main game loop.
    pub fn start(self) -> ! {
        // If a startup handler is set, call it. This is where you could initialize your game.
        self.startup.map(|f| f());

        // Initialize the interrupt descriptor table (IDT) with the configured handlers.
        interrupts::init_idt(self);
        // Initialize the PICs to handle hardware interrupts.
        unsafe { interrupts::PICS.lock().initialize() };
        // Enable hardware interrupts.
        x86_64::instructions::interrupts::enable();

        // Enter the main game loop or CPU idle loop.
        (self.cpu_loop)();
    }

    // Setter methods for configuring each type of handler follow, including timer, keyboard, startup, and CPU loop.

    // Handles timer interrupts, such as for game tick updates.
    pub fn handle_timer(&self) {
        if let Some(timer) = self.timer {
            (timer)()
        }
    }

    // Handles keyboard interrupts, translating them into game controls.
    pub fn handle_keyboard(&self, key: DecodedKey) {
        if let Some(keyboard) = self.keyboard {
            (keyboard)(key)
        }
    }

    // Additional methods to set each handler in the table, following the Builder pattern.
    // Each setter returns Self to allow chaining calls.
}

// A simple loop that halts the CPU to wait for the next interrupt, conserving power.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// The panic handler is invoked on unrecoverable errors. It logs the panic info and enters the halt loop.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    hlt_loop();
}

// Additional structs or modules (like RacyCell) for concurrency or unsafe operations can be defined below.

