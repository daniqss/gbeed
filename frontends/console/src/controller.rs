use gbeed_core::{prelude::DMG_SCREEN_WIDTH, Controller, Ppu, Renderer, SerialListener};
use gbeed_raylib_common::{color, Texture};
use raylib::prelude::*;

pub struct ConsoleController {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,
    pub screen: Texture,
    pub palette: color::Palette,
    pub palette_color: color::PaletteColor,
}

impl Renderer for ConsoleController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        ((self.screen[index] as u32) << 16)
            | ((self.screen[index + 1] as u32) << 8)
            | (self.screen[index + 2] as u32)
    }

    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let shade = (palette >> (color_id * 2)) & 0x03;
        let color = self.palette_color[shade as usize];

        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        self.screen[index] = color.r;
        self.screen[index + 1] = color.g;
        self.screen[index + 2] = color.b;
    }

    fn update_screen(&mut self, _: &Ppu) { self.screen.update(); }
}

impl SerialListener for ConsoleController {
    fn on_transfer(&mut self, data: u8) {
        println!("through serial port -> {data:04X}");
    }
}

impl Controller for ConsoleController {}
