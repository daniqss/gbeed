const DMG_SCREEN_WIDTH: usize = 160;
const DMG_SCREEN_HEIGHT: usize = 144;

#[derive(Debug)]
pub struct Display {
    framebuffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    double_buffer: [[u32; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
    bg_buffer: [[u32; 256]; 256],
}

impl Default for Display {
    fn default() -> Self {
        Self {
            framebuffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            double_buffer: [[0; DMG_SCREEN_WIDTH]; DMG_SCREEN_HEIGHT],
            bg_buffer: [[0; 256]; 256],
        }
    }
}
