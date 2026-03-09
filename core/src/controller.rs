use crate::ppu::DefaultRenderer;
use crate::ppu::Renderer;
use crate::serial::DefaultSerialListener;
use crate::serial::SerialListener;

pub trait Controller: SerialListener + Renderer {}

pub struct DefaultController {
    renderer: DefaultRenderer,
    serial: DefaultSerialListener,
}

impl DefaultController {
    pub fn new() -> Self {
        Self {
            renderer: DefaultRenderer::new(),
            serial: DefaultSerialListener,
        }
    }
}

impl Default for DefaultController {
    fn default() -> Self { Self::new() }
}

impl Renderer for DefaultController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }
    fn write_pixel(&mut self, x: usize, y: usize, color: u32) { self.renderer.write_pixel(x, y, color) }
    fn draw_screen(&mut self) { self.renderer.draw_screen() }
}

impl SerialListener for DefaultController {
    fn on_transfer(&mut self, data: u8) { self.serial.on_transfer(data) }
}

impl Controller for DefaultController {}
