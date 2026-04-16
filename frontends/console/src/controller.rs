use gbeed_core::{AudioPlayer, Controller, Ppu, Renderer, SerialListener, prelude::DMG_SCREEN_WIDTH};
use gbeed_raylib_common::{
    Texture, color,
    settings::{SpeedUpMode, SpeedUpMultiplier, TargetedFps},
};
use raylib::prelude::*;

const SAMPLE_RATE: u32 = 44100;
const BUFFER_SIZE: usize = 4096;

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
    _audio: &'a RaylibAudio,
    audio_stream: AudioStream<'a>,
    audio_buffer: Vec<i16>,
}

impl<'a> ConsoleController<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        audio: &'a RaylibAudio,
        screen: Texture,
        palette: color::Palette,
    ) -> Self {
        let palette_color = palette.get_palette_color();

        let audio_stream = audio.new_audio_stream(SAMPLE_RATE, 16, 1);
        audio_stream.play();

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
            audio_buffer: Vec::with_capacity(BUFFER_SIZE * 2),
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

    fn stereo(&self) -> bool { false }

    fn write_buffer(&mut self, samples: &[i16]) {
        self.audio_buffer.extend_from_slice(samples);

        while self.audio_stream.is_processed() && !self.audio_buffer.is_empty() {
            let to_write = self.audio_buffer.len().min(BUFFER_SIZE);
            let chunk: Vec<i16> = self.audio_buffer.drain(..to_write).collect();
            if let Err(e) = self.audio_stream.update(&chunk) {
                eprintln!("update error: {e}");
            }
        }

        if self.audio_buffer.len() > BUFFER_SIZE * 8 {
            self.audio_buffer
                .drain(..self.audio_buffer.len() - BUFFER_SIZE * 8);
        }
    }
}

impl Controller for ConsoleController<'_> {}
