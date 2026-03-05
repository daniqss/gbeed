use gbeed_core::prelude::*;
use gbeed_core::Renderer;
use raylib::ffi::PixelFormat;
use raylib::prelude::*;

// we should distinguish between desktop arm and armv6 32 bits of the raspberry pi zero
#[cfg(target_arch = "arm")]
const SCREEN_WIDTH: i32 = 400;
#[cfg(not(target_arch = "arm"))]
const SCREEN_WIDTH: i32 = 1920;
#[cfg(target_arch = "arm")]
const SCREEN_HEIGHT: i32 = 240;
#[cfg(not(target_arch = "arm"))]
const SCREEN_HEIGHT: i32 = 1080;
#[cfg(target_arch = "arm")]
const WINDOW_TITLE: &str = "gbeed";
#[cfg(not(target_arch = "arm"))]
const WINDOW_TITLE: &str = "gbeed -- desktop";

pub struct RaylibRenderer {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,
    texture: Texture2D,
    pub screen: [u8; DMG_SCREEN_WIDTH * DMG_SCREEN_HEIGHT * 3],
}

impl RaylibRenderer {
    pub fn new() -> Self {
        let (mut rl, thread) = raylib::init()
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title(WINDOW_TITLE)
            .resizable()
            .build();

        let mut frame_image =
            Image::gen_image_color(DMG_SCREEN_WIDTH as i32, DMG_SCREEN_HEIGHT as i32, Color::BLACK);
        frame_image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
        let texture = rl
            .load_texture_from_image(&thread, &frame_image)
            .expect("Failed to load texture");

        Self {
            rl,
            thread,
            texture,
            screen: [0; DMG_SCREEN_WIDTH * DMG_SCREEN_HEIGHT * 3],
        }
    }
}

impl Renderer for RaylibRenderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let i = (y * DMG_SCREEN_WIDTH + x) * 3;
        let r = self.screen[i] as u32;
        let g = self.screen[i + 1] as u32;
        let b = self.screen[i + 2] as u32;
        (r << 16) | (g << 8) | b
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        let i = (y * DMG_SCREEN_WIDTH + x) * 3;
        self.screen[i] = ((color >> 16) & 0xFF) as u8;
        self.screen[i + 1] = ((color >> 8) & 0xFF) as u8;
        self.screen[i + 2] = (color & 0xFF) as u8;
    }

    fn draw_screen(&mut self) {
        let _ = self.texture.update_texture(&self.screen);

        let thread = &self.thread;
        let texture = &self.texture;

        let mut d = self.rl.begin_drawing(thread);
        d.clear_background(Color::BLACK);

        let screen_w = d.get_screen_width() as f32;
        let screen_h = d.get_screen_height() as f32;
        let scale = (screen_w / DMG_SCREEN_WIDTH as f32).min(screen_h / DMG_SCREEN_HEIGHT as f32);

        let dest_w = DMG_SCREEN_WIDTH as f32 * scale;
        let dest_h = DMG_SCREEN_HEIGHT as f32 * scale;
        let dest_x = (screen_w - dest_w) / 2.0;
        let dest_y = (screen_h - dest_h) / 2.0;

        d.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(dest_x, dest_y, dest_w, dest_h),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        d.draw_fps(10, screen_h as i32 - 20);
    }
}
