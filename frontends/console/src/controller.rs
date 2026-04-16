use gbeed_core::{AudioPlayer, Controller, Ppu, Renderer, SerialListener, prelude::DMG_SCREEN_WIDTH};
use gbeed_raylib_common::{
    Texture, color,
    settings::{SpeedUpMode, SpeedUpMultiplier, TargetedFps},
};
use raylib::ffi;
use raylib::prelude::*;

const SAMPLE_RATE: u32 = 44100;

pub struct ConsoleController<'a> {
    pub screen: Texture,
    pub palette: color::Palette,
    pub palette_color: color::PaletteColor,
    pub speed_up_mode: SpeedUpMode,
    pub speed_up_multiplier: SpeedUpMultiplier,
    pub targeted_fps: TargetedFps,
    pub draw_debug_info: bool,

    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    _audio: RaylibAudio,
    audio_stream: ffi::AudioStream,
    audio_buffer: Vec<i16>,
}

impl<'a> ConsoleController<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        screen: Texture,
        palette: color::Palette,
    ) -> Self {
        let palette_color = palette.get_palette_color();

        let audio = RaylibAudio::init_audio_device().expect("Failed to init audio");
        let audio_stream = unsafe { ffi::LoadAudioStream(SAMPLE_RATE, 16, 2) };

        unsafe {
            ffi::PlayAudioStream(audio_stream);
        }

        Self {
            screen,
            palette,
            palette_color,
            speed_up_mode: SpeedUpMode::default(),
            speed_up_multiplier: SpeedUpMultiplier::default(),
            targeted_fps: TargetedFps::default(),
            draw_debug_info: false,
            rl,
            thread,
            _audio: audio,
            audio_stream,
            audio_buffer: Vec::with_capacity(8192),
        }
    }
}

impl Renderer for ConsoleController<'_> {
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

impl SerialListener for ConsoleController<'_> {
    fn on_transfer(&mut self, data: u8) {
        println!("through serial port -> {data:04X}");
    }
}

impl AudioPlayer for ConsoleController<'_> {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }

    fn stereo(&self) -> bool { true }

    fn write_buffer(&mut self, samples: &[i16]) {
        let stream = self.audio_stream;

        self.audio_buffer.extend_from_slice(samples);
        unsafe {
            while ffi::IsAudioStreamProcessed(stream) && !self.audio_buffer.is_empty() {
                let frame_count = (self.audio_buffer.len() / 2) as i32;

                let to_write = frame_count.min(4096);
                ffi::UpdateAudioStream(stream, self.audio_buffer.as_ptr() as *const _, to_write);
                self.audio_buffer.drain(0..(to_write as usize * 2));
            }

            // if buffer is still too large, cap it to avoid infinite growth/latency
            if self.audio_buffer.len() > 32768 {
                self.audio_buffer.drain(0..(self.audio_buffer.len() - 32768));
            }
        }
    }
}

impl Controller for ConsoleController<'_> {}
