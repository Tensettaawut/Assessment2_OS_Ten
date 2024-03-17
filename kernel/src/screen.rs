// Original code from rust-osdev/bootloader crate https://github.com/rust-osdev/bootloader

use core::{fmt, ptr};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use kernel::RacyCell;

static WRITER: RacyCell<Option<ScreenWriter>> = RacyCell::new(None);
pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let writer = unsafe { WRITER.get_mut() }.as_mut().unwrap();
        writer.write_str(s)
    }
}

pub fn screenwriter() -> &'static mut ScreenWriter {
    let writer = unsafe { WRITER.get_mut() }.as_mut().unwrap();
    writer
}

pub fn init(buffer: &'static mut FrameBuffer) {
    let info = buffer.info();
    let framebuffer = buffer.buffer_mut();
    let writer = ScreenWriter::new(framebuffer, info);
    *unsafe { WRITER.get_mut() } = Some(writer);
}

/// Additional vertical space between lines
const LINE_SPACING: usize = 0;

pub struct ScreenWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl ScreenWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        logger.clear();
        logger
    }

    /// Clears the screen.
    pub fn clear(&mut self) {
        self.framebuffer.fill(0);
    }

    /// Draws a rectangle on the screen.
    /// `x` and `y` specify the top left corner of the rectangle.
    /// `width` and `height` specify the dimensions of the rectangle.
    /// `r`, `g`, `b` specify the color of the rectangle.
    pub fn draw_rectangle(&mut self, x: usize, y: usize, width: usize, height: usize, r: u8, g: u8, b: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.draw_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }

    /// Draws a pixel on the screen at (x, y) with the specified color.
    pub fn draw_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= self.info.width.into() || y >= self.info.height.into() {
            return; // Out of bounds
        }

        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            _ => return, // Unsupported pixel format
        };
        let byte_offset = pixel_offset * self.info.bytes_per_pixel;
        let bytes = &mut self.framebuffer[byte_offset..(byte_offset + self.info.bytes_per_pixel)];
        bytes.copy_from_slice(&color);
    }
}

unsafe impl Send for ScreenWriter {}
unsafe impl Sync for ScreenWriter {}

impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // For simplicity, this method remains focused on text rendering, which might be useful for debug output.
        Ok(())
    }
}
