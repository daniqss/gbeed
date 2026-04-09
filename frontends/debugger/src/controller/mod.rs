mod listener;
pub mod renderer;

use gbeed_core::{Controller, Renderer, SerialListener};
use raylib::prelude::*;

use listener::RaylibSerialListener;
use renderer::{Layout, RaylibRenderer};

pub struct RaylibController {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,
    pub renderer: RaylibRenderer,
    pub serial_listener: RaylibSerialListener,
}

impl RaylibController {
    pub fn new(mut rl: RaylibHandle, thread: RaylibThread, layout: Layout) -> Self {
        let renderer = RaylibRenderer::new(&mut rl, &thread, layout);
        Self {
            rl,
            thread,
            renderer,
            serial_listener: RaylibSerialListener,
        }
    }
}

impl Renderer for RaylibController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }
    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        self.renderer.write_pixel(x, y, palette, color_id);
    }
    fn draw_screen(&mut self) {
        self.renderer.draw_screen(&mut self.rl, &self.thread);
    }
}

impl SerialListener for RaylibController {
    fn on_transfer(&mut self, data: u8) { self.serial_listener.on_transfer(data) }
}

impl Controller for RaylibController {}
