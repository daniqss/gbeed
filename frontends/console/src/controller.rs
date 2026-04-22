use gbeed_core::{
    AudioPlayer, Controller, Ppu, Renderer, SAMPLE_RATE, STEREO_BUFFER_SIZE, SerialListener,
    prelude::DMG_SCREEN_WIDTH,
};
use gbeed_raylib_common::{
    Texture, color,
    settings::{SpeedUpMode, SpeedUpMultiplier, TargetedFps},
};
use raylib::prelude::*;

pub struct ConsoleController<'a> {
    pub screen: Texture,
    pub palette: color::Palette,
    pub palette_color: color::PaletteColor,

    sample_idx: usize,
    audio_buffer: Box<[i16; STEREO_BUFFER_SIZE]>,
    audio_stream: AudioStream<'a>,

    pub speed_up_mode: SpeedUpMode,
    pub speed_up_multiplier: SpeedUpMultiplier,
    pub targeted_fps: TargetedFps,
    pub draw_debug_info: bool,

    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    _audio: &'a RaylibAudio,
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

            sample_idx: 0,
            audio_buffer: Box::new([0; STEREO_BUFFER_SIZE]),
            audio_stream,

            speed_up_mode: SpeedUpMode::default(),
            speed_up_multiplier: SpeedUpMultiplier::default(),
            targeted_fps: TargetedFps::default(),
            draw_debug_info: false,

            rl,
            thread,
            _audio: audio,
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
    fn playing_stereo(&self) -> bool { false }

    fn push_sample(&mut self, left: i16, right: i16) {
        self.audio_buffer[self.sample_idx] = left;
        self.audio_buffer[self.sample_idx + 1] = right;
        self.sample_idx = (self.sample_idx + 2) % STEREO_BUFFER_SIZE;
    }

    fn flush_buffer(&mut self) {
        while self.audio_stream.is_processed() && self.sample_idx > 0 {
            if let Err(e) = self.audio_stream.update(&self.audio_buffer[..self.sample_idx]) {
                eprintln!("update error: {e}");
            }
            self.sample_idx = 0;
        }
    }
}

impl Controller for ConsoleController<'_> {}
