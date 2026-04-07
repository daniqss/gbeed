use crate::ppu::DefaultRenderer;
use crate::ppu::Renderer;
use crate::serial::DefaultSerialListener;
use crate::serial::SerialListener;

pub trait Controller: SerialListener + Renderer {}

#[macro_export]
macro_rules! controller {
    ($name:ident, $listener:ty, $renderer:ty) => {
        pub struct $name {
            listener: $listener,
            renderer: $renderer,
        }

        impl Renderer for $name {
            fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }

            fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
                self.renderer.write_pixel(x, y, palette, color_id)
            }
        }

        impl SerialListener for $name {
            fn on_transfer(&mut self, data: u8) { self.listener.on_transfer(data) }
        }

        impl Controller for $name {}
    };
}

controller!(DefaultController, DefaultSerialListener, DefaultRenderer);

impl DefaultController {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            listener: DefaultSerialListener::new(),
            renderer: DefaultRenderer::new(),
        }
    }
}
