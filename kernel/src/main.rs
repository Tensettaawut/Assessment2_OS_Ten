#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

mod screen;
mod allocator;
mod interrupts;

use alloc::vec::Vec;
use core::fmt::Write;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping::Dynamic;
use kernel::{HandlerTable, serial};
use pc_keyboard::{DecodedKey, KeyCode};
use crate::screen::{Writer, screenwriter};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Dynamic);
    config.kernel_stack_size = 256 * 1024;
    config
};
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

static mut PLAYER_POSITION: usize = 50;
static mut BULLETS: Vec<usize> = Vec::new();
static mut ENEMIES: Vec<usize> = Vec::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    writeln!(serial(), "Space Invaders Game Starting...").unwrap();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        screen::init(framebuffer);
    } else {
        panic!("Framebuffer not available");
    }

    let physical_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let usable_region = boot_info.memory_regions.iter().filter(|r| r.kind == MemoryRegionKind::Usable).last().unwrap();
    allocator::init_heap((physical_offset + usable_region.start) as usize);

    HandlerTable::new()
        .keyboard(handle_key)
        .timer(handle_tick)
        .startup(game_startup)
        .cpu_loop(game_loop)
        .start();
}

fn game_startup() {
    writeln!(Writer, "Game Initializing...").unwrap();
    // Initialize player, enemies, and bullets.
    unsafe {
        PLAYER_POSITION = screenwriter().info().width as usize / 2; // Center the player
        BULLETS.clear();
        ENEMIES.clear();
        for i in (0..screenwriter().info().width).step_by(20) {
            ENEMIES.push(i as usize);
        }
    }
    draw_game();
}

fn game_loop() -> ! {
    loop {
        // Main game loop here. This is a placeholder for a more complex game logic.
        x86_64::instructions::hlt();
    }
}

fn handle_tick() {
    // Move bullets
    unsafe {
        BULLETS.retain(|x| {
            *x = x.saturating_sub(1);
            *x > 0
        });
    }

    // Simple enemy movement (for demonstration)
    unsafe {
        for x in ENEMIES.iter_mut() {
            *x = (*x + 1) % screenwriter().info().width as usize;
        }
    }

    draw_game();
}

fn handle_key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(' ') => shoot_bullet(),
        DecodedKey::Unicode('a') | DecodedKey::RawKey(KeyCode::ArrowLeft) => move_player(-1),
        DecodedKey::Unicode('d') | DecodedKey::RawKey(KeyCode::ArrowRight) => move_player(1),
        _ => {}
    }
}

fn shoot_bullet() {
    unsafe {
        let pos = PLAYER_POSITION;
        BULLETS.push(pos);
    }
}

fn move_player(direction: isize) {
    unsafe {
        let new_pos = (PLAYER_POSITION as isize + direction).max(0).min(screenwriter().info().width as isize - 1) as usize;
        PLAYER_POSITION = new_pos;
    }
    draw_game();
}

fn draw_game() {
    screenwriter().clear();

    // Draw player
    unsafe {
        screenwriter().draw_rectangle(PLAYER_POSITION, screenwriter().info().height as usize - 20, 10, 10, 0xff, 0xff, 0xff);
    }

    // Draw bullets
    unsafe {
        for &bullet in BULLETS.iter() {
            screenwriter().draw_rectangle(bullet, screenwriter().info().height as usize - 40, 2, 5, 0xff, 0x00, 0x00);
        }
    }

    // Draw enemies
    unsafe {
        for &enemy in ENEMIES.iter() {
            screenwriter().draw_rectangle(enemy, 20, 10, 10, 0x00, 0xff, 0x00);
        }
    }
}
