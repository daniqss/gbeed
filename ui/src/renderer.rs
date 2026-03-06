use gbeed_core::prelude::*;
use gbeed_core::Renderer;
use raylib::ffi::PixelFormat;
use raylib::prelude::*;

// gb screen: scale 3 → 480×432
const GB_SCALE: i32 = 3;
const GB_W: i32 = DMG_SCREEN_WIDTH as i32 * GB_SCALE;
const GB_H: i32 = DMG_SCREEN_HEIGHT as i32 * GB_SCALE;

const PAD: i32 = 16;
const HEADER_H: i32 = 38;
// x where the right panel begins
const RIGHT_X: i32 = PAD + GB_W + PAD * 2;

// tile viewer: 128 tiles in a 16×8 grid, native 8×8 px, upscaled ×3
const T_COLS: i32 = 16;
const T_ROWS: i32 = 8;
const T_PX: i32 = 8;
const T_SCALE: i32 = 3;
const T_TEX_W: i32 = T_COLS * T_PX; // 128
const T_TEX_H: i32 = T_ROWS * T_PX; // 64
const TV_W: i32 = T_TEX_W * T_SCALE; // 384
const TV_H: i32 = T_TEX_H * T_SCALE; // 192

const C_BG: Color = Color {
    r: 8,
    g: 10,
    b: 16,
    a: 255,
};
const C_PANEL: Color = Color {
    r: 15,
    g: 19,
    b: 29,
    a: 255,
};
const C_BORDER: Color = Color {
    r: 30,
    g: 40,
    b: 56,
    a: 255,
};
const C_ACCENT: Color = Color {
    r: 68,
    g: 210,
    b: 84,
    a: 255,
};
const C_ACCENT_DIM: Color = Color {
    r: 24,
    g: 58,
    b: 30,
    a: 255,
};
const C_TEXT: Color = Color {
    r: 216,
    g: 238,
    b: 216,
    a: 255,
};
const C_SUB: Color = Color {
    r: 80,
    g: 116,
    b: 82,
    a: 255,
};
const C_DIVIDER: Color = Color {
    r: 28,
    g: 36,
    b: 50,
    a: 255,
};

// classic dmg 4-shade palette (0 = lightest, 3 = darkest)
const GB_PAL: [Color; 4] = [
    Color {
        r: 230,
        g: 252,
        b: 214,
        a: 255,
    },
    Color {
        r: 136,
        g: 194,
        b: 112,
        a: 255,
    },
    Color {
        r: 48,
        g: 106,
        b: 80,
        a: 255,
    },
    Color {
        r: 8,
        g: 18,
        b: 22,
        a: 255,
    },
];

// ─────────────────────────────────────────────────────────────────────────────

pub struct Texture {
    pub texture: Texture2D,
    pub pixels: Vec<u8>,
}

impl Texture {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let mut img = Image::gen_image_color(width, height, Color::BLACK);
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
                    let c = GB_PAL[color_idx as usize];
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
        let bx = RIGHT_X + 156;
        let by = PAD + 14;
        let bw = 118_i32;
        let bh = 26_i32;
        let mp = self.rl.get_mouse_position();
        self.rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && (mp.x as i32) >= bx
            && (mp.x as i32) < bx + bw
            && (mp.y as i32) >= by
            && (mp.y as i32) < by + bh
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

        // borrow distinct fields before begin_drawing takes &mut self.rl
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

        d.clear_background(C_BG);
        d.draw_rectangle(RIGHT_X - PAD, 0, 1, sh, C_DIVIDER);

        // LEFT PANEL
        let gx = PAD;

        // header: game name + region above the screen
        d.draw_text(&game_name, gx, PAD, 22, C_TEXT);
        d.draw_text(&format!("{game_region}"), gx, PAD + 24, 10, C_SUB);

        // gb screen with accent border
        let gy = PAD + HEADER_H;
        d.draw_rectangle(gx - 3, gy - 3, GB_W + 6, GB_H + 6, C_ACCENT_DIM);
        d.draw_rectangle_lines(gx - 3, gy - 3, GB_W + 6, GB_H + 6, C_ACCENT);
        d.draw_texture_pro(
            screen_tex,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(gx as f32, gy as f32, GB_W as f32, GB_H as f32),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // controls
        let ctrl_y = gy + GB_H + PAD * 2;
        d.draw_text("controls", gx, ctrl_y, 10, C_SUB);

        // d-pad
        let dpx = gx + 78;
        let dpy = ctrl_y + 72;
        let bs = 34_i32;
        let gap = 3_i32;
        draw_pad_btn(
            &mut d,
            dpx - bs / 2,
            dpy - bs - gap,
            bs,
            bs,
            "W",
            "UP",
            buttons.up,
        );
        draw_pad_btn(&mut d, dpx - bs / 2, dpy + gap, bs, bs, "S", "DN", buttons.down);
        draw_pad_btn(
            &mut d,
            dpx - bs - gap,
            dpy - bs / 2,
            bs,
            bs,
            "A",
            "LT",
            buttons.left,
        );
        draw_pad_btn(&mut d, dpx + gap, dpy - bs / 2, bs, bs, "D", "RT", buttons.right);
        // centre cap
        d.draw_rectangle(dpx - bs / 2, dpy - bs / 2, bs, bs, C_PANEL);
        d.draw_rectangle_lines(dpx - bs / 2, dpy - bs / 2, bs, bs, C_BORDER);

        // a/b (diagonal like the real dmg)
        let abx = gx + 340;
        let aby = ctrl_y + 24;
        let abs = 36_i32;
        draw_action_btn(&mut d, abx + 48, aby, abs, "A", "Z / J", buttons.a);
        draw_action_btn(&mut d, abx, aby + 44, abs, "B", "X / K", buttons.b);

        // start / select
        let ss_x = gx + 148;
        let ss_y = ctrl_y + 106;
        draw_small_btn(&mut d, ss_x, ss_y, 60, 20, "SELECT", "SHIFT / ;", buttons.select);
        draw_small_btn(&mut d, ss_x + 78, ss_y, 60, 20, "START", "L", buttons.start);

        // RIGHT PANEL
        let rx = RIGHT_X;

        // fps counter + cycle button
        let fps_val = d.get_fps();
        d.draw_text("frame rate", rx, PAD, 10, C_SUB);
        d.draw_text(&format!("{fps_val:3}"), rx, PAD + 14, 26, C_TEXT);
        d.draw_text("fps", rx + 48, PAD + 22, 12, C_SUB);

        let fb_x = rx + 156;
        let fb_y = PAD + 14;
        let fb_w = 118_i32;
        let fb_h = 26_i32;
        d.draw_rectangle(fb_x, fb_y, fb_w, fb_h, C_PANEL);
        d.draw_rectangle_lines(fb_x, fb_y, fb_w, fb_h, C_ACCENT);
        let tw = d.measure_text(target_str, 12);
        d.draw_text(target_str, fb_x + (fb_w - tw) / 2, fb_y + 7, 12, C_ACCENT);

        // three vram tile viewers
        const TV_LABELS: [&str; 3] = [
            "vram  $8000-$87ff  (block 0)",
            "vram  $8800-$8fff  (block 1)",
            "vram  $9000-$97ff  (block 2)",
        ];
        let tv_start_y = PAD + 50;
        let tv_stride = 12 + TV_H + 14;

        for i in 0..3_usize {
            let ty = tv_start_y + i as i32 * tv_stride;
            let tty = ty + 12;

            d.draw_text(TV_LABELS[i], rx, ty, 10, C_SUB);
            d.draw_rectangle(rx - 2, tty - 2, TV_W + 4, TV_H + 4, C_PANEL);
            d.draw_rectangle_lines(rx - 2, tty - 2, TV_W + 4, TV_H + 4, C_BORDER);
            d.draw_texture_pro(
                tile_texs[i],
                Rectangle::new(0.0, 0.0, T_TEX_W as f32, T_TEX_H as f32),
                Rectangle::new(rx as f32, tty as f32, TV_W as f32, TV_H as f32),
                Vector2::ZERO,
                0.0,
                Color::WHITE,
            );

            // tile grid overlay
            let grid_col = Color {
                r: 0,
                g: 0,
                b: 0,
                a: 60,
            };
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
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    key: &str,
    hint: &str,
    pressed: bool,
) {
    let (bg, fg, bd) = if pressed {
        (C_ACCENT, C_BG, C_ACCENT)
    } else {
        (C_PANEL, C_TEXT, C_BORDER)
    };
    d.draw_rectangle(x, y, w, h, bg);
    d.draw_rectangle_lines(x, y, w, h, bd);
    let fs = 14;
    let tw = d.measure_text(key, fs);
    d.draw_text(key, x + (w - tw) / 2, y + (h - fs) / 2 - 3, fs, fg);
    let hs = 8;
    let hw = d.measure_text(hint, hs);
    d.draw_text(
        hint,
        x + (w - hw) / 2,
        y + (h - fs) / 2 + fs - 1,
        hs,
        if pressed { C_BG } else { C_SUB },
    );
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
        (C_ACCENT, C_BG, C_ACCENT)
    } else {
        (C_PANEL, C_TEXT, C_BORDER)
    };
    d.draw_rectangle(x, y, size, size, bg);
    d.draw_rectangle_lines(x, y, size, size, bd);
    let fs = 18;
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (size - tw) / 2, y + (size - fs) / 2, fs, fg);
    let ks = 8;
    let kw = d.measure_text(key, ks);
    d.draw_text(key, x + (size - kw) / 2, y + size + 4, ks, C_SUB);
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
        (C_ACCENT, C_BG, C_ACCENT)
    } else {
        (C_PANEL, C_TEXT, C_BORDER)
    };
    d.draw_rectangle(x, y, w, h, bg);
    d.draw_rectangle_lines(x, y, w, h, bd);
    let fs = 9;
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (w - tw) / 2, y + (h - fs) / 2, fs, fg);
    d.draw_text(key, x, y + h + 3, 7, C_SUB);
}
