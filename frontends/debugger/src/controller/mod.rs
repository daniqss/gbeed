mod listener;
pub mod renderer;

use gbeed_core::{Controller, Ppu, Renderer, SerialListener};
use raylib::prelude::*;

use listener::DebuggerSerialListener;
use renderer::{DebuggerRenderer, Layout};

pub struct DebuggerController {
    pub renderer: DebuggerRenderer,
    pub serial_listener: DebuggerSerialListener,
    pub rl: RaylibHandle,
    pub thread: RaylibThread,
}

impl DebuggerController {
    pub fn new(mut rl: RaylibHandle, thread: RaylibThread, layout: Layout) -> Self {
        Self {
            renderer: DebuggerRenderer::new(&mut rl, &thread, layout),
            serial_listener: DebuggerSerialListener,
            rl,
            thread,
        }
    }
}

impl Renderer for DebuggerController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }
    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        self.renderer.write_pixel(x, y, palette, color_id);
    }
    fn update_screen(&mut self, ppu: &Ppu) { self.renderer.update_screen(ppu) }
}

impl SerialListener for DebuggerController {
    fn on_transfer(&mut self, data: u8) { self.serial_listener.on_transfer(data) }
}

impl Controller for DebuggerController {}
