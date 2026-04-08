use crate::ppu::{DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH, Ppu};

/// UI crates that uses gbeed must implement
pub trait Renderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32;
    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8);
    fn update_screen(&mut self, ppu: &Ppu);
}

pub struct DefaultRenderer {
    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    colors: [u32; 4],
}

impl DefaultRenderer {
    pub fn new() -> Self {
        Self {
            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            colors: [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F],
        }
    }
}

impl Default for DefaultRenderer {
    fn default() -> Self { Self::new() }
}

impl Renderer for DefaultRenderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.framebuffer[y][x] }
    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let shade = (palette >> (color_id * 2)) & 0x03;
        self.framebuffer[y][x] = self.colors[shade as usize];
    }
    fn update_screen(&mut self, _: &Ppu) {}
}
