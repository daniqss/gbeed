use gbeed_core::prelude::*;
use gbeed_core::Renderer;
use raylib::ffi::PixelFormat;
use raylib::prelude::*;

use crate::colors;

const GB_SCALE: i32 = 4;
const GB_W: i32 = DMG_SCREEN_WIDTH as i32 * GB_SCALE;
const GB_H: i32 = DMG_SCREEN_HEIGHT as i32 * GB_SCALE;

const PAD: i32 = 16;
const HEADER_H: i32 = 34;
const RIGHT_X: i32 = PAD + GB_W + PAD * 2;

const T_COLS: i32 = 16;
const T_ROWS: i32 = 8;
const T_PX: i32 = 8;
const T_SCALE: i32 = 3;
const T_TEX_W: i32 = T_COLS * T_PX;
const T_TEX_H: i32 = T_ROWS * T_PX;
const TV_W: i32 = T_TEX_W * T_SCALE;
const TV_H: i32 = T_TEX_H * T_SCALE;

pub struct Texture {
    pub texture: Texture2D,
    pub pixels: Vec<u8>,
}

impl Texture {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let mut img = Image::gen_image_color(width, height, colors::BACKGROUND);
        img.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
        let texture = rl.load_texture_from_image(thread, &img).unwrap();
        let pixels = vec![0u8; (width * height * 3) as usize];
        Self { texture, pixels }
    }

    pub fn update(&mut self) {
        if let Err(e) = self.texture.update_texture(&self.pixels) {
            eprintln!("texture update failed: {e:?}");
        }
    }
}

#[derive(Copy, Clone)]
pub enum FpsMode {
    Target60,
    Target120,
    Unlimited,
}

#[derive(Default)]
pub struct ButtonStates {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

pub struct RaylibRenderer {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,

    pub screen_texture: Texture,
    pub bg_map_texture: Texture,
    pub tile_textures: [Texture; 3],

    pub buttons: ButtonStates,
    pub game_name: String,
    pub game_region: String,
    pub fps_mode: FpsMode,
}

impl RaylibRenderer {
    pub fn new() -> Self {
        let (mut rl, thread) = raylib::init().size(1280, 720).title("gbeed").resizable().build();
        rl.set_target_fps(60);

        let screen_texture = Texture::new(
            &mut rl,
            &thread,
            DMG_SCREEN_WIDTH as i32,
            DMG_SCREEN_HEIGHT as i32,
        );
        let bg_map_texture = Texture::new(&mut rl, &thread, 256, 256);
        let tile_textures = [
            Texture::new(&mut rl, &thread, T_TEX_W, T_TEX_H),
            Texture::new(&mut rl, &thread, T_TEX_W, T_TEX_H),
            Texture::new(&mut rl, &thread, T_TEX_W, T_TEX_H),
        ];

        Self {
            rl,
            thread,
            screen_texture,
            bg_map_texture,
            tile_textures,
            buttons: ButtonStates::default(),
            game_name: "Unknown".into(),
            game_region: "Unknown".into(),
            fps_mode: FpsMode::Target60,
        }
    }

    pub fn set_game_info(&mut self, name: impl Into<String>, region: impl Into<String>) {
        let clean = |s: String| s.chars().filter(|c| *c != '\0' && !c.is_control()).collect();
        self.game_name = clean(name.into());
        self.game_region = clean(region.into());
    }

    // decodes a 2bpp vram block into the tile texture (region 0/$8000, 1/$8800, 2/$9000)
    pub fn update_tiles(&mut self, region: usize, data: &[u8]) {
        debug_assert!(region < 3);
        let tex = &mut self.tile_textures[region];
        let stride = T_TEX_W as usize;

        for tile in 0..128_usize {
            let tile_base_x = (tile % T_COLS as usize) * T_PX as usize;
            let tile_base_y = (tile / T_COLS as usize) * T_PX as usize;
            let data_base = tile * 16;

            for row in 0..8_usize {
                let lo = data.get(data_base + row * 2).copied().unwrap_or(0);
                let hi = data.get(data_base + row * 2 + 1).copied().unwrap_or(0);
                for col in 0..8_usize {
                    let bit = 7 - col;
                    let color_idx = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                    let c = colors::GB_PALETTE[color_idx as usize];
                    let px = tile_base_x + col;
                    let py = tile_base_y + row;
                    let i = (py * stride + px) * 3;
                    tex.pixels[i] = c.r;
                    tex.pixels[i + 1] = c.g;
                    tex.pixels[i + 2] = c.b;
                }
            }
        }
        tex.update();
    }

    pub fn cycle_fps(&mut self) {
        self.fps_mode = match self.fps_mode {
            FpsMode::Target60 => {
                self.rl.set_target_fps(120);
                FpsMode::Target120
            }
            FpsMode::Target120 => {
                self.rl.set_target_fps(0);
                FpsMode::Unlimited
            }
            FpsMode::Unlimited => {
                self.rl.set_target_fps(60);
                FpsMode::Target60
            }
        };
    }

    pub fn fps_btn_clicked(&self) -> bool {
        // button coords must stay in sync with draw_screen
        let fb_w = 118_i32;
        let fb_h = 26_i32;
        let fb_x = PAD + GB_W - fb_w;
        let header_cy = PAD + HEADER_H / 2;
        let fb_y = header_cy - fb_h / 2;
        let mp = self.rl.get_mouse_position();
        self.rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && (mp.x as i32) >= fb_x
            && (mp.x as i32) < fb_x + fb_w
            && (mp.y as i32) >= fb_y
            && (mp.y as i32) < fb_y + fb_h
    }
}

impl Renderer for RaylibRenderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let i = (y * DMG_SCREEN_WIDTH + x) * 3;
        ((self.screen_texture.pixels[i] as u32) << 16)
            | ((self.screen_texture.pixels[i + 1] as u32) << 8)
            | (self.screen_texture.pixels[i + 2] as u32)
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        let i = (y * DMG_SCREEN_WIDTH + x) * 3;
        self.screen_texture.pixels[i] = ((color >> 16) & 0xFF) as u8;
        self.screen_texture.pixels[i + 1] = ((color >> 8) & 0xFF) as u8;
        self.screen_texture.pixels[i + 2] = (color & 0xFF) as u8;
    }

    fn draw_screen(&mut self) {
        self.screen_texture.update();

        let thread = &self.thread;
        let screen_tex = &self.screen_texture.texture;
        let tile_texs = [
            &self.tile_textures[0].texture,
            &self.tile_textures[1].texture,
            &self.tile_textures[2].texture,
        ];
        let buttons = &self.buttons;
        let game_name = self.game_name.clone();
        let game_region = self.game_region.clone();
        let target_str = match self.fps_mode {
            FpsMode::Target60 => "TARGET  60 Hz",
            FpsMode::Target120 => "TARGET 120 Hz",
            FpsMode::Unlimited => "TARGET  UNLIM",
        };

        let mut d = self.rl.begin_drawing(thread);
        let _sw = d.get_screen_width();
        let sh = d.get_screen_height();

        d.clear_background(colors::BACKGROUND);

        // vertical divider spanning full height
        d.draw_rectangle(RIGHT_X - PAD, 0, 1, sh, colors::SECONDARY);

        // LEFT PANEL

        let gx = PAD;

        // header vertically centred around header_cy
        let header_cy = PAD + HEADER_H / 2;

        let title_fs = 22;
        let title_y = header_cy - title_fs / 2;
        let name_w = d.measure_text(&game_name, title_fs);
        d.draw_text(&game_name, gx, title_y, title_fs, colors::FOREGROUND);

        let region_fs = 11;
        let region_y = header_cy - region_fs / 2;
        d.draw_text(
            &game_region,
            gx + name_w + 10,
            region_y,
            region_fs,
            colors::SECONDARY,
        );

        let fps_fs = 26;
        let fps_val = d.get_fps();
        let fps_str = format!("{fps_val:3}");
        let fps_label_fs = 11;
        let fps_label = "fps";
        let fps_num_w = d.measure_text(&fps_str, fps_fs);
        let fps_label_w = d.measure_text(fps_label, fps_label_fs);

        let fb_w = 118_i32;
        let fb_h = 26_i32;
        let fb_x = gx + GB_W - fb_w;
        let fb_y = header_cy - fb_h / 2;

        let fps_group_w = fps_num_w + 4 + fps_label_w;
        let fps_x = fb_x - 16 - fps_group_w;
        let fps_y = header_cy - fps_fs / 2;
        let fps_label_y = header_cy - fps_label_fs / 2;
        d.draw_text(&fps_str, fps_x, fps_y, fps_fs, colors::FOREGROUND);
        d.draw_text(
            fps_label,
            fps_x + fps_num_w + 4,
            fps_label_y,
            fps_label_fs,
            colors::SECONDARY,
        );

        d.draw_rectangle(fb_x, fb_y - 4, fb_w, fb_h, colors::BACKGROUND);
        d.draw_rectangle_lines(fb_x, fb_y - 4, fb_w, fb_h, colors::PRIMARY);
        let tw = d.measure_text(target_str, 12);
        d.draw_text(
            target_str,
            fb_x + (fb_w - tw) / 2,
            fb_y + (fb_h - 15) / 2,
            12,
            colors::PRIMARY,
        );

        // gb screen starts immediately after header
        let gy = PAD + HEADER_H;
        d.draw_rectangle(gx - 3, gy - 3, GB_W + 6, GB_H + 6, colors::PRIMARY);
        d.draw_rectangle_lines(gx - 3, gy - 3, GB_W + 6, GB_H + 6, colors::PRIMARY);
        d.draw_texture_pro(
            screen_tex,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(gx as f32, gy as f32, GB_W as f32, GB_H as f32),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // controls centred under the gb screen
        // screen spans gx..gx+GB_W, centre = gx + GB_W/2
        let ctrl_y = gy + GB_H + PAD * 2;
        let screen_cx = gx + GB_W / 2;

        // dpad: centre the cross on screen_cx - 160 (leave room for a/b on the right)
        let dpx = screen_cx - 160;
        let dpy = ctrl_y + 50;
        let arm = 28_i32;
        let s = 17_i32;

        // apex toward center: up=0°, down=180°, left=270°, right=90°
        draw_pad_btn(&mut d, dpx, dpy - arm, s, 0.0, "W", buttons.up);
        draw_pad_btn(&mut d, dpx, dpy + arm, s, 180.0, "S", buttons.down);
        draw_pad_btn(&mut d, dpx - arm, dpy, s, 270.0, "A", buttons.left);
        draw_pad_btn(&mut d, dpx + arm, dpy, s, 90.0, "D", buttons.right);

        // start / select centred between dpad and a/b
        let ss_cx = screen_cx;
        let ss_w = 60_i32;
        let ss_gap = 18_i32;
        let ss_total = ss_w * 2 + ss_gap;
        let ss_x = ss_cx - ss_total / 2;
        let ss_y = dpy - 10;
        draw_small_btn(
            &mut d,
            ss_x,
            ss_y,
            ss_w,
            20,
            "SELECT",
            "SHIFT / ;",
            buttons.select,
        );
        draw_small_btn(
            &mut d,
            ss_x + ss_w + ss_gap,
            ss_y,
            ss_w,
            20,
            "START",
            "L",
            buttons.start,
        );

        // a/b: centre on screen_cx + 160
        let abx = screen_cx + 160;
        let aby = ctrl_y + 24;
        let abs = 36_i32;
        draw_action_btn(&mut d, abx - abs / 2 + 24, aby, abs, "A", "Z / J", buttons.a);
        draw_action_btn(&mut d, abx - abs / 2 - 24, aby + 44, abs, "B", "X / K", buttons.b);

        // RIGHT PANEL
        let rx = RIGHT_X;

        const TV_LABELS: [&str; 3] = [
            "vram  $8000-$87ff  (block 0)",
            "vram  $8800-$8fff  (block 1)",
            "vram  $9000-$97ff  (block 2)",
        ];
        let tv_start_y = PAD;
        let tv_stride = 12 + TV_H + 14;

        for i in 0..3_usize {
            let ty = tv_start_y + i as i32 * tv_stride;
            let tty = ty + 12;

            d.draw_text(TV_LABELS[i], rx, ty, 10, colors::SECONDARY);
            d.draw_rectangle(rx - 2, tty - 2, TV_W + 4, TV_H + 4, colors::BACKGROUND);
            d.draw_rectangle_lines(rx - 2, tty - 2, TV_W + 4, TV_H + 4, colors::BACKGROUND);
            d.draw_texture_pro(
                tile_texs[i],
                Rectangle::new(0.0, 0.0, T_TEX_W as f32, T_TEX_H as f32),
                Rectangle::new(rx as f32, tty as f32, TV_W as f32, TV_H as f32),
                Vector2::ZERO,
                0.0,
                Color::WHITE,
            );

            let mut grid_col = colors::BACKGROUND;
            grid_col.a = 60;
            let cell = T_PX * T_SCALE;
            for col in 0..=T_COLS {
                d.draw_line(rx + col * cell, tty, rx + col * cell, tty + TV_H, grid_col);
            }
            for row in 0..=T_ROWS {
                d.draw_line(rx, tty + row * cell, rx + TV_W, tty + row * cell, grid_col);
            }
        }
    }
}

fn draw_pad_btn(
    d: &mut RaylibDrawHandle,
    cx: i32,
    cy: i32,
    s: i32,
    rotation_deg: f32,
    key: &str,
    pressed: bool,
) {
    let (bg, fg, bd) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };

    let sf = s as f32;
    let base: [(f32, f32); 5] = [(-sf, -sf), (sf, -sf), (sf, 0.0), (0.0, sf), (-sf, 0.0)];

    let rad = rotation_deg.to_radians();
    let (sin_r, cos_r) = rad.sin_cos();

    let verts: Vec<Vector2> = base
        .iter()
        .map(|&(x, y)| {
            Vector2::new(
                cx as f32 + x * cos_r - y * sin_r,
                cy as f32 + x * sin_r + y * cos_r,
            )
        })
        .collect();

    // fan of 3 triangles, CCW winding in screen space
    d.draw_triangle(verts[0], verts[3], verts[1], bg); // A D B
    d.draw_triangle(verts[0], verts[4], verts[3], bg); // A E D
    d.draw_triangle(verts[1], verts[3], verts[2], bg); // B D C

    // outline
    for i in 0..5 {
        d.draw_line_v(verts[i], verts[(i + 1) % 5], bd);
    }

    let fs = 11;
    let tw = d.measure_text(key, fs);
    d.draw_text(key, cx - tw / 2, cy - fs / 2, fs, fg);
}

fn draw_action_btn(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    size: i32,
    label: &str,
    key: &str,
    pressed: bool,
) {
    let (bg, fg, bd) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };
    d.draw_rectangle(x, y, size, size, bg);
    d.draw_rectangle_lines(x, y, size, size, bd);
    let fs = 18;
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (size - tw) / 2, y + (size - fs) / 2, fs, fg);
    let ks = 8;
    let kw = d.measure_text(key, ks);
    d.draw_text(key, x + (size - kw) / 2, y + size + 4, ks, colors::SECONDARY);
}

fn draw_small_btn(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    key: &str,
    pressed: bool,
) {
    let (bg, fg, bd) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };
    d.draw_rectangle(x, y, w, h, bg);
    d.draw_rectangle_lines(x, y, w, h, bd);
    let fs = 9;
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (w - tw) / 2, y + (h - fs) / 2, fs, fg);
    d.draw_text(key, x, y + h + 3, 7, colors::SECONDARY);
}
