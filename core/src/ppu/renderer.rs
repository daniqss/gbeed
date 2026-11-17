use crate::ppu::Ppu;

/// UI crates that uses gbeed must implement
#[cfg_attr(feature = "ffi", gbeed_macros::ffi_callbacks)]
pub trait Renderer {
    fn read_pixel(&self, _x: usize, _y: usize) -> u32 { 0xFF }
    /// Called for every pixel drawn, its shade (0..=3) is `(palette >> (color_id * 2)) & 3`
    fn write_pixel(&mut self, _x: usize, _y: usize, _palette: u8, _color_id: u8) {}
    /// Called once per frame, when the PPU enters VBlank
    fn update_screen(&mut self, _ppu: &Ppu) {}
}
