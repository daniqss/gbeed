use crate::ppu::Ppu;

/// UI crates that uses gbeed must implement
pub trait Renderer {
    fn read_pixel(&self, _x: usize, _y: usize) -> u32 { 0xFF }
    fn write_pixel(&mut self, _x: usize, _y: usize, _palette: u8, _color_id: u8) {}
    fn update_screen(&mut self, _ppu: &Ppu) {}
}
