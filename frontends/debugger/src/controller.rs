use gbeed_core::prelude::*;
use gbeed_core::AudioPlayer;
use gbeed_raylib_common::{color::DMG_CLASSIC_PALETTE, settings::SpeedUpMultiplier, Texture};
use raylib::ffi;
use raylib::prelude::*;

pub const TILES_PER_ROW: i32 = 16;
pub const TILES_PER_COLUMN: i32 = 8;
pub const TILE_PIXEL_SIZE: i32 = 8;
pub const TILE_DISPLAY_SCALE: i32 = 3;
pub const TILE_TEXTURE_WIDTH: i32 = TILES_PER_ROW * TILE_PIXEL_SIZE;
pub const TILE_TEXTURE_HEIGHT: i32 = TILES_PER_COLUMN * TILE_PIXEL_SIZE;
pub const TILE_DISPLAY_WIDTH: i32 = TILE_TEXTURE_WIDTH * TILE_DISPLAY_SCALE;
pub const TILE_DISPLAY_HEIGHT: i32 = TILE_TEXTURE_HEIGHT * TILE_DISPLAY_SCALE;

const SAMPLE_RATE: u32 = 44100;

pub struct DebuggerController<'a> {
    pub screen_texture: Texture,
    pub tile_textures: [Texture; 3],
    pub bg_map_texture: Texture,

    pub scroll_x: i32,
    pub scroll_y: i32,

    pub speed_up_multiplier: SpeedUpMultiplier,

    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    audio: Option<RaylibAudio>,
    audio_stream: Option<ffi::AudioStream>,
}

impl<'a> DebuggerController<'a> {
    pub fn new(rl: &'a mut RaylibHandle, thread: &'a RaylibThread) -> Self {
        Self {
            screen_texture: Texture::new(rl, thread, DMG_SCREEN_WIDTH as i32, DMG_SCREEN_HEIGHT as i32),
            tile_textures: [
                Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
                Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
                Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
            ],
            bg_map_texture: Texture::new(rl, thread, 256, 256),

            scroll_x: 0,
            scroll_y: 0,
            speed_up_multiplier: SpeedUpMultiplier::OneAndHalf,

            rl,
            thread,
            audio: None,
            audio_stream: None,
        }
    }

    pub fn init_audio(&mut self) {
        self.audio = Some(RaylibAudio::init_audio_device().expect("Failed to init audio"));
        self.audio_stream = Some(unsafe { ffi::LoadAudioStream(SAMPLE_RATE, 16, 2) });

        if let Some(stream) = self.audio_stream {
            unsafe {
                ffi::PlayAudioStream(stream);
            }
        }
    }
}

impl Renderer for DebuggerController<'_> {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        ((self.screen_texture[index] as u32) << 16)
            | ((self.screen_texture[index + 1] as u32) << 8)
            | (self.screen_texture[index + 2] as u32)
    }

    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;
        let shade = (palette >> (color_id * 2)) & 0x03;
        let color = DMG_CLASSIC_PALETTE[shade as usize];

        self.screen_texture[index] = color.r;
        self.screen_texture[index + 1] = color.g;
        self.screen_texture[index + 2] = color.b;
    }

    fn update_screen(&mut self, ppu: &Ppu) {
        self.screen_texture.update();

        update_tiles(&mut self.tile_textures[0], ppu.tile_block0());
        update_tiles(&mut self.tile_textures[1], ppu.tile_block1());
        update_tiles(&mut self.tile_textures[2], ppu.tile_block2());

        update_bg_map(
            &mut self.bg_map_texture,
            ppu.bg_map0(),
            ppu.tile_data(),
            ppu.bg_tile_map_address(),
            ppu.get_bg_palette(),
        );

        update_scroll(&mut self.scroll_x, &mut self.scroll_y, ppu.get_scroll());
    }
}

impl SerialListener for DebuggerController<'_> {
    fn on_transfer(&mut self, data: u8) {
        println!("through serial port -> {data:04X}");
    }
}

impl Controller for DebuggerController<'_> {}

impl AudioPlayer for DebuggerController<'_> {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }

    fn stereo(&self) -> bool { true }

    fn write_buffer(&mut self, samples: &[i16]) {
        if let Some(stream) = self.audio_stream {
            let frame_count = (samples.len() / 2) as i32;
            if frame_count > 0 {
                unsafe {
                    if ffi::IsAudioStreamProcessed(stream) {
                        ffi::UpdateAudioStream(stream, samples.as_ptr() as *const _, frame_count);
                    }
                }
            }
        }
    }
}

pub fn update_bg_map(
    texture: &mut Texture,
    map_data: &[u8],
    tile_data: &[u8],
    is_mode_8000: bool,
    palette: u8,
) {
    for ty in 0..32 {
        for tx in 0..32 {
            let tn = map_data[ty * 32 + tx];
            let base = if is_mode_8000 {
                tn as usize * 16
            } else {
                (0x1000_i32 + (tn as i8 as i32) * 16) as usize
            };
            for row in 0..8 {
                let lb = tile_data[base + row * 2];
                let hb = tile_data[base + row * 2 + 1];
                for col in 0..8 {
                    let bit = 7 - col;
                    let cid = (((hb >> bit) & 1) << 1) | ((lb >> bit) & 1);
                    let color = DMG_CLASSIC_PALETTE[((palette >> (cid * 2)) & 0x03) as usize];
                    let idx = ((ty * 8 + row) * 256 + (tx * 8 + col)) * 3;
                    texture[idx] = color.r;
                    texture[idx + 1] = color.g;
                    texture[idx + 2] = color.b;
                }
            }
        }
    }
    texture.update();
}

pub fn update_tiles(texture: &mut Texture, data: &[u8]) {
    for ti in 0..128 {
        let bx = (ti % TILES_PER_ROW as usize) * 8;
        let by = (ti / TILES_PER_ROW as usize) * 8;
        for row in 0..8 {
            let lb = data[ti * 16 + row * 2];
            let hb = data[ti * 16 + row * 2 + 1];
            for col in 0..8 {
                let bit = 7 - col;
                let cid = (((hb >> bit) & 1) << 1) | ((lb >> bit) & 1);
                let color = DMG_CLASSIC_PALETTE[cid as usize];
                let idx = ((by + row) * TILE_TEXTURE_WIDTH as usize + (bx + col)) * 3;
                texture[idx] = color.r;
                texture[idx + 1] = color.g;
                texture[idx + 2] = color.b;
            }
        }
    }
    texture.update();
}

pub fn update_scroll(scroll_x: &mut i32, scroll_y: &mut i32, scroll: (u8, u8)) {
    (*scroll_x, *scroll_y) = (scroll.0 as i32, scroll.1 as i32);
}
