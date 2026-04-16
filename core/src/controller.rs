use crate::apu::{AudioPlayer, DefaultAudioPlayer};
use crate::ppu::{DefaultRenderer, Ppu, Renderer};
use crate::serial::{DefaultSerialListener, SerialListener};

pub trait Controller: SerialListener + Renderer + AudioPlayer {}

#[macro_export]
macro_rules! controller {
    ($name:ident, $listener:ty, $renderer:ty, $audio_player:ty) => {
        pub struct $name {
            listener: $listener,
            renderer: $renderer,
            audio_player: $audio_player,
        }

        impl Renderer for $name {
            fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }

            fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
                self.renderer.write_pixel(x, y, palette, color_id)
            }

            fn update_screen(&mut self, ppu: &Ppu) { self.renderer.update_screen(ppu) }
        }

        impl SerialListener for $name {
            fn on_transfer(&mut self, data: u8) { self.listener.on_transfer(data) }
        }

        impl AudioPlayer for $name {
            fn sample_rate(&self) -> u32 { self.audio_player.sample_rate() }
            fn stereo(&self) -> bool { self.audio_player.stereo() }
            fn write_buffer(&mut self, samples: &[i16]) { self.audio_player.write_buffer(samples) }
        }

        impl Controller for $name {}
    };
}

controller!(
    DefaultController,
    DefaultSerialListener,
    DefaultRenderer,
    DefaultAudioPlayer
);

impl DefaultController {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            listener: DefaultSerialListener::new(),
            renderer: DefaultRenderer::new(),
            audio_player: DefaultAudioPlayer::new(),
        }
    }
}
