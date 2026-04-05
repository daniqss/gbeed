mod listener;
pub mod renderer;

use gbeed_core::{Controller, Renderer, SerialListener};
use raylib::prelude::*;

use listener::RaylibSerialListener;
use renderer::RaylibRenderer;

pub struct RaylibController {
    pub renderer: RaylibRenderer,
    serial_listener: RaylibSerialListener,
}

impl RaylibController {
    pub fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
        Self {
            renderer: RaylibRenderer::new(rl, thread),
            serial_listener: RaylibSerialListener,
        }
    }
}

impl Renderer for RaylibController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }
    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        self.renderer.write_pixel(x, y, palette, color_id);
    }
    fn draw_screen(&mut self) { self.renderer.draw_screen() }
}

impl SerialListener for RaylibController {
    fn on_transfer(&mut self, data: u8) { self.serial_listener.on_transfer(data) }
}

impl Controller for RaylibController {}
