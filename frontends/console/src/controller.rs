use gbeed_core::{prelude::DMG_SCREEN_WIDTH, Controller, Renderer, SerialListener};
use gbeed_raylib_common::texture::Texture;
use raylib::prelude::*;

pub const GB_PALETTE: [Color; 4] = [
    Color {
        r: 196,
        g: 207,
        b: 161,
        a: 255,
    },
    Color {
        r: 139,
        g: 149,
        b: 109,
        a: 255,
    },
    Color {
        r: 77,
        g: 83,
        b: 60,
        a: 255,
    },
    Color {
        r: 31,
        g: 31,
        b: 31,
        a: 255,
    },
];

pub struct ConsoleController {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,
    pub screen: Texture,
}

impl Renderer for ConsoleController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        ((self.screen[index] as u32) << 16)
            | ((self.screen[index + 1] as u32) << 8)
            | (self.screen[index + 2] as u32)
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        self.screen[index] = ((color >> 16) & 0xFF) as u8;
        self.screen[index + 1] = ((color >> 8) & 0xFF) as u8;
        self.screen[index + 2] = (color & 0xFF) as u8;
    }

    fn get_color(&self, palette: u8, color_id: u8) -> u32 {
        let shade = (palette >> (color_id * 2)) & 0x03;
        let color = GB_PALETTE[shade as usize];

        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }

    fn draw_screen(&mut self) { self.screen.update(); }
}

impl SerialListener for ConsoleController {
    fn on_transfer(&mut self, data: u8) {
        println!("through serial port -> {data:04X}");
    }
}

impl Controller for ConsoleController {}
