use crate::ppu::{DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH};

/// UI crates that uses gbeed must implement
pub trait Renderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32;
    fn write_pixel(&mut self, x: usize, y: usize, color: u32);
    fn draw_screen(&mut self);
}

pub struct DefaultRenderer {
    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
}

impl DefaultRenderer {
    pub fn new() -> Self {
        Self {
            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
        }
    }
}

impl Renderer for DefaultRenderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.framebuffer[y][x] }
    fn write_pixel(&mut self, x: usize, y: usize, color: u32) { self.framebuffer[y][x] = color; }
    fn draw_screen(&mut self) {}
}
