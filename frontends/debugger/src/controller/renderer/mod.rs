use gbeed_core::Ppu;
use gbeed_core::Renderer;
use gbeed_core::prelude::*;
use gbeed_raylib_common::color::DMG_CLASSIC_PALETTE;
use gbeed_raylib_common::{Texture, input, settings};
use raylib::prelude::*;

pub const FOREGROUND: Color = DMG_CLASSIC_PALETTE[0];
pub const SECONDARY: Color = DMG_CLASSIC_PALETTE[1];
pub const PRIMARY: Color = DMG_CLASSIC_PALETTE[2];
pub const BACKGROUND: Color = DMG_CLASSIC_PALETTE[3];

pub const PANEL_PADDING: i32 = 16;
pub const HEADER_HEIGHT: i32 = 34;

pub const TILES_PER_ROW: i32 = 16;
pub const TILES_PER_COLUMN: i32 = 8;
pub const TILE_PIXEL_SIZE: i32 = 8;
pub const TILE_DISPLAY_SCALE: i32 = 3;
pub const TILE_TEXTURE_WIDTH: i32 = TILES_PER_ROW * TILE_PIXEL_SIZE;
pub const TILE_TEXTURE_HEIGHT: i32 = TILES_PER_COLUMN * TILE_PIXEL_SIZE;
pub const TILE_DISPLAY_WIDTH: i32 = TILE_TEXTURE_WIDTH * TILE_DISPLAY_SCALE;
pub const TILE_DISPLAY_HEIGHT: i32 = TILE_TEXTURE_HEIGHT * TILE_DISPLAY_SCALE;

#[derive(Clone, Copy)]
pub struct Layout {
    pub is_mobile: bool,
    pub game_x: i32,
    pub game_y: i32,
    pub scaled_screen_width: i32,
    pub scaled_screen_height: i32,
    pub screen_center_x: i32,
    pub controls_y: i32,
    pub middle_panel_x: i32,
    pub right_panel_x: i32,
    pub bg_map_width: i32,
    pub bg_map_height: i32,

    pub dpad_x: i32,
    pub dpad_y: i32,
    pub dpad_arm: i32,
    pub dpad_size: i32,

    pub start_select_x: i32,
    pub start_select_y: i32,
    pub start_select_width: i32,

    pub action_buttons_x: i32,
    pub action_buttons_y: i32,
    pub action_button_size: i32,
}

impl Layout {
    pub fn new(screen_width: i32, _screen_height: i32, is_mobile: bool) -> Self {
        let game_y = PANEL_PADDING + HEADER_HEIGHT;

        if is_mobile {
            let game_x = 0;
            let scaled_screen_width = screen_width;
            let scaled_screen_height = screen_width * DMG_SCREEN_HEIGHT as i32 / DMG_SCREEN_WIDTH as i32;
            let screen_center_x = screen_width / 2;

            let controls_y = game_y + scaled_screen_height + PANEL_PADDING * 4;

            let dpad_arm = 70;
            let dpad_size = 45;
            let dpad_x = screen_width / 4;
            let dpad_y = controls_y + 130;

            let action_button_size = 85;
            let action_buttons_x = screen_width * 3 / 4;
            let action_buttons_y = controls_y + 80;

            let start_select_width = 100;
            let start_select_gap = 25;
            let start_select_total = start_select_width * 2 + start_select_gap;
            let start_select_x = screen_center_x - start_select_total / 2;
            let start_select_y = dpad_y + dpad_arm + 60;

            Self {
                is_mobile,
                game_x,
                game_y,
                scaled_screen_width,
                scaled_screen_height,
                screen_center_x,
                controls_y,
                middle_panel_x: 0,
                right_panel_x: 0,
                bg_map_width: 0,
                bg_map_height: 0,
                dpad_x,
                dpad_y,
                dpad_arm,
                dpad_size,
                start_select_x,
                start_select_y,
                start_select_width,
                action_buttons_x,
                action_buttons_y,
                action_button_size,
            }
        } else {
            let screen_scale = 4;
            let scaled_screen_width = DMG_SCREEN_WIDTH as i32 * screen_scale;
            let scaled_screen_height = DMG_SCREEN_HEIGHT as i32 * screen_scale;

            let game_x = PANEL_PADDING;
            let screen_center_x = game_x + scaled_screen_width / 2;
            let controls_y = game_y + scaled_screen_height + PANEL_PADDING * 2;

            let middle_panel_x = PANEL_PADDING + scaled_screen_width + PANEL_PADDING * 2;
            let bg_map_width = scaled_screen_height;
            let bg_map_height = scaled_screen_height;
            let right_panel_x = middle_panel_x + bg_map_width + PANEL_PADDING * 2;

            let dpad_arm = 28;
            let dpad_size = 17;
            let dpad_x = screen_center_x - 160;
            let dpad_y = controls_y + 50;

            let start_select_width = 60;
            let start_select_gap = 18;
            let start_select_total = start_select_width * 2 + start_select_gap;
            let start_select_x = screen_center_x - start_select_total / 2;
            let start_select_y = dpad_y - 10;

            let action_button_size = 36;
            let action_buttons_x = screen_center_x + 160;
            let action_buttons_y = controls_y + 24;

            Self {
                is_mobile,
                game_x,
                game_y,
                scaled_screen_width,
                scaled_screen_height,
                screen_center_x,
                controls_y,
                middle_panel_x,
                right_panel_x,
                bg_map_width,
                bg_map_height,
                dpad_x,
                dpad_y,
                dpad_arm,
                dpad_size,
                start_select_x,
                start_select_y,
                start_select_width,
                action_buttons_x,
                action_buttons_y,
                action_button_size,
            }
        }
    }

    pub fn get_mouse_triggers(&self) -> input::InputMouseTriggers {
        use input::MouseButtonArea;

        input::InputMouseTriggers {
            up: MouseButtonArea::new(
                self.dpad_x - self.dpad_size,
                self.dpad_y - self.dpad_arm - self.dpad_size,
                self.dpad_size * 2,
                self.dpad_size * 2,
            ),
            down: MouseButtonArea::new(
                self.dpad_x - self.dpad_size,
                self.dpad_y + self.dpad_arm - self.dpad_size,
                self.dpad_size * 2,
                self.dpad_size * 2,
            ),
            left: MouseButtonArea::new(
                self.dpad_x - self.dpad_arm - self.dpad_size,
                self.dpad_y - self.dpad_size,
                self.dpad_size * 2,
                self.dpad_size * 2,
            ),
            right: MouseButtonArea::new(
                self.dpad_x + self.dpad_arm - self.dpad_size,
                self.dpad_y - self.dpad_size,
                self.dpad_size * 2,
                self.dpad_size * 2,
            ),
            select: MouseButtonArea::new(
                self.start_select_x,
                self.start_select_y,
                self.start_select_width,
                20,
            ),
            start: MouseButtonArea::new(
                self.start_select_x + self.start_select_width + 18,
                self.start_select_y,
                self.start_select_width,
                20,
            ),
            a: MouseButtonArea::new(
                self.action_buttons_x - self.action_button_size / 2 + 24,
                self.action_buttons_y,
                self.action_button_size,
                self.action_button_size,
            ),
            b: MouseButtonArea::new(
                self.action_buttons_x - self.action_button_size / 2 - 24,
                self.action_buttons_y + self.action_button_size + 8,
                self.action_button_size,
                self.action_button_size,
            ),
            speed_up: if self.is_mobile {
                None
            } else {
                Some(MouseButtonArea::new(
                    self.screen_center_x - 118 / 2,
                    self.controls_y - 20,
                    118,
                    26,
                ))
            },
            ..Default::default()
        }
    }
}

pub struct DebuggerRenderer {
    pub screen_texture: Texture,
    pub bg_map_texture: Texture,
    pub tile_textures: [Texture; 3],

    pub buttons: input::InputState,
    pub game_name: String,
    pub game_region: String,
    pub fps_mode: settings::TargetedFps,

    pub scroll_x: i32,
    pub scroll_y: i32,
    pub layout: Layout,
}

impl DebuggerRenderer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, layout: Layout) -> Self {
        let screen_texture = Texture::new(rl, thread, DMG_SCREEN_WIDTH as i32, DMG_SCREEN_HEIGHT as i32);
        let bg_map_texture = Texture::new(rl, thread, 256, 256);
        let tile_textures = [
            Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
            Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
            Texture::new(rl, thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
        ];

        Self {
            screen_texture,
            bg_map_texture,
            tile_textures,
            buttons: input::InputState::default(),
            game_name: "Unknown".into(),
            game_region: "Unknown".into(),
            fps_mode: settings::TargetedFps::Target60,
            scroll_x: 0,
            scroll_y: 0,
            layout,
        }
    }

    pub fn set_game_info(&mut self, name: impl Into<String>, region: impl std::fmt::Debug) {
        let clean = |s: String| s.chars().filter(|c| *c != '\0' && !c.is_control()).collect();
        self.game_name = clean(name.into());
        self.game_region = clean(format!("{region:?}"));
    }

    pub fn update_scroll(&mut self, scroll: (i32, i32)) { (self.scroll_x, self.scroll_y) = scroll; }

    pub fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;
        ((self.screen_texture[index] as u32) << 16)
            | ((self.screen_texture[index + 1] as u32) << 8)
            | (self.screen_texture[index + 2] as u32)
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;
        let shade = (palette >> (color_id * 2)) & 0x03;
        let color = DMG_CLASSIC_PALETTE[shade as usize];

        self.screen_texture[index] = color.r;
        self.screen_texture[index + 1] = color.g;
        self.screen_texture[index + 2] = color.b;
    }
}

impl DebuggerRenderer {
    pub fn draw_screen(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.screen_texture.update();

        let screen_texture = &self.screen_texture.texture;
        let tile_textures = [
            &self.tile_textures[0].texture,
            &self.tile_textures[1].texture,
            &self.tile_textures[2].texture,
        ];

        let buttons = &self.buttons;
        let game_name = self.game_name.clone();
        let game_region = self.game_region.clone();
        let layout = self.layout;

        let mut d = rl.begin_drawing(thread);
        d.clear_background(BACKGROUND);

        if !layout.is_mobile {
            d.draw_rectangle(
                layout.middle_panel_x - PANEL_PADDING,
                0,
                1,
                d.get_screen_height(),
                SECONDARY,
            );
        }

        // Header
        let game_x = layout.game_x;
        let header_center_y = PANEL_PADDING + HEADER_HEIGHT / 2;
        let title_font_size = 22;
        let title_y = header_center_y - title_font_size / 2;
        let name_width = d.measure_text(&game_name, title_font_size);
        d.draw_text(&game_name, game_x, title_y, title_font_size, FOREGROUND);

        let region_font_size = 11;
        let region_y = header_center_y - region_font_size / 2;
        d.draw_text(
            &game_region,
            game_x + name_width + 10,
            region_y,
            region_font_size,
            SECONDARY,
        );

        let fps_font_size = 26;
        let fps_str = format!("{:3}", d.get_fps());
        let fps_number_width = d.measure_text(&fps_str, fps_font_size);
        let fps_x = game_x + layout.scaled_screen_width - (fps_number_width + 30);
        d.draw_text(
            &fps_str,
            fps_x,
            header_center_y - fps_font_size / 2,
            fps_font_size,
            FOREGROUND,
        );
        d.draw_text(
            "fps",
            fps_x + fps_number_width + 4,
            header_center_y - 5,
            11,
            SECONDARY,
        );

        // Screen
        let game_y = layout.game_y;
        d.draw_rectangle(
            game_x - 3,
            game_y - 3,
            layout.scaled_screen_width + 6,
            layout.scaled_screen_height + 6,
            PRIMARY,
        );
        d.draw_texture_pro(
            screen_texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(
                game_x as f32,
                game_y as f32,
                layout.scaled_screen_width as f32,
                layout.scaled_screen_height as f32,
            ),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // Controls
        // #[cfg(not(target_arch = "wasm32"))]
        // if !layout.is_mobile {
        //     let screen_center_x = layout.screen_center_x;
        //     let controls_y = layout.controls_y;
        //     draw_fps_btn(
        //         &mut d,
        //         screen_center_x,
        //         controls_y,
        //         match self.fps_mode {
        //             FpsMode::Target60 => "TARGET  60 Hz",
        //             FpsMode::Target120 => "TARGET 120 Hz",
        //             FpsMode::Unlimited => "TARGET  UNLIM",
        //         },
        //     );
        // }

        // D-PAD
        let dpad_x = layout.dpad_x;
        let dpad_y = layout.dpad_y;
        let dpad_arm = layout.dpad_arm;
        let dpad_size = layout.dpad_size;
        draw_pad_btn(
            &mut d,
            dpad_x,
            dpad_y - dpad_arm,
            dpad_size,
            0.0,
            "W",
            buttons.up,
            layout.is_mobile,
        );
        draw_pad_btn(
            &mut d,
            dpad_x,
            dpad_y + dpad_arm,
            dpad_size,
            180.0,
            "S",
            buttons.down,
            layout.is_mobile,
        );
        draw_pad_btn(
            &mut d,
            dpad_x - dpad_arm,
            dpad_y,
            dpad_size,
            270.0,
            "A",
            buttons.left,
            layout.is_mobile,
        );
        draw_pad_btn(
            &mut d,
            dpad_x + dpad_arm,
            dpad_y,
            dpad_size,
            90.0,
            "D",
            buttons.right,
            layout.is_mobile,
        );

        // Start/Select
        draw_small_btn(
            &mut d,
            layout.start_select_x,
            layout.start_select_y,
            layout.start_select_width,
            20,
            "SELECT",
            "SHIFT / ;",
            buttons.select,
            layout.is_mobile,
        );
        draw_small_btn(
            &mut d,
            layout.start_select_x + layout.start_select_width + 18,
            layout.start_select_y,
            layout.start_select_width,
            20,
            "START",
            "L",
            buttons.start,
            layout.is_mobile,
        );

        // A/B
        let action_x = layout.action_buttons_x;
        let action_y = layout.action_buttons_y;
        let action_size = layout.action_button_size;
        draw_action_btn(
            &mut d,
            action_x - action_size / 2 + 24,
            action_y,
            action_size,
            "A",
            "Z / J",
            buttons.a,
            layout.is_mobile,
        );
        draw_action_btn(
            &mut d,
            action_x - action_size / 2 - 24,
            action_y + action_size + 8,
            action_size,
            "B",
            "X / K",
            buttons.b,
            layout.is_mobile,
        );

        // Debug Panels
        if !layout.is_mobile {
            self.draw_debug_panels(&mut d, layout, tile_textures);
        }
    }

    fn draw_debug_panels(
        &self,
        d: &mut RaylibDrawHandle,
        layout: Layout,
        tile_textures: [&raylib::texture::Texture2D; 3],
    ) {
        let game_y = layout.game_y;
        let map_x = layout.middle_panel_x;
        let bg_map_y = game_y;

        d.draw_text("bg map $9800", map_x, bg_map_y - 12, 14, SECONDARY);
        d.draw_texture_pro(
            &self.bg_map_texture.texture,
            Rectangle::new(0.0, 0.0, 256.0, 256.0),
            Rectangle::new(
                map_x as f32,
                bg_map_y as f32,
                layout.bg_map_width as f32,
                layout.bg_map_height as f32,
            ),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // Scroll overlay
        let scale = layout.bg_map_width as f32 / 256.0;
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;
        let scroll_end_x = (scroll_x + 160) % 256;
        let scroll_end_y = (scroll_y + 144) % 256;
        let scroll_wraps_x = (scroll_x + 160) >= 256;
        let scroll_wraps_y = (scroll_y + 144) >= 256;

        let to_screen = |px: i32, py: i32| {
            Vector2::new(
                map_x as f32 + px as f32 * scale,
                bg_map_y as f32 + py as f32 * scale,
            )
        };
        const OVERLAY_COLOR: Color = Color {
            r: 255,
            g: 220,
            b: 0,
            a: 255,
        };

        for &ry in &[scroll_y, scroll_end_y] {
            if !scroll_wraps_x {
                d.draw_line_ex(
                    to_screen(scroll_x, ry),
                    to_screen(scroll_x + 160, ry),
                    1.5,
                    OVERLAY_COLOR,
                );
            } else {
                d.draw_line_ex(to_screen(scroll_x, ry), to_screen(256, ry), 1.5, OVERLAY_COLOR);
                d.draw_line_ex(to_screen(0, ry), to_screen(scroll_end_x, ry), 1.5, OVERLAY_COLOR);
            }
        }
        for &cx in &[scroll_x, scroll_end_x] {
            if !scroll_wraps_y {
                d.draw_line_ex(
                    to_screen(cx, scroll_y),
                    to_screen(cx, scroll_y + 144),
                    1.5,
                    OVERLAY_COLOR,
                );
            } else {
                d.draw_line_ex(to_screen(cx, scroll_y), to_screen(cx, 256), 1.5, OVERLAY_COLOR);
                d.draw_line_ex(to_screen(cx, 0), to_screen(cx, scroll_end_y), 1.5, OVERLAY_COLOR);
            }
        }

        // Tiles
        let right_x = layout.right_panel_x;
        const TV_LABELS: [&str; 3] = [
            "vram  $8000-$87ff  (block 0)",
            "vram  $8800-$8fff  (block 1)",
            "vram  $9000-$97ff  (block 2)",
        ];
        let tv_stride = TILE_DISPLAY_HEIGHT + 16;

        for i in 0..3 {
            let ty = game_y + i as i32 * tv_stride;
            d.draw_text(TV_LABELS[i], right_x, ty - 12, 14, SECONDARY);
            d.draw_texture_pro(
                tile_textures[i],
                Rectangle::new(0.0, 0.0, TILE_TEXTURE_WIDTH as f32, TILE_TEXTURE_HEIGHT as f32),
                Rectangle::new(
                    right_x as f32,
                    ty as f32,
                    TILE_DISPLAY_WIDTH as f32,
                    TILE_DISPLAY_HEIGHT as f32,
                ),
                Vector2::ZERO,
                0.0,
                Color::WHITE,
            );

            let mut grid_color = BACKGROUND;
            grid_color.a = 60;
            let cell = TILE_PIXEL_SIZE * TILE_DISPLAY_SCALE;
            for col in 0..=TILES_PER_ROW {
                d.draw_line(
                    right_x + col * cell,
                    ty,
                    right_x + col * cell,
                    ty + TILE_DISPLAY_HEIGHT,
                    grid_color,
                );
            }
            for row in 0..=TILES_PER_COLUMN {
                d.draw_line(
                    right_x,
                    ty + row * cell,
                    right_x + TILE_DISPLAY_WIDTH,
                    ty + row * cell,
                    grid_color,
                );
            }
        }
    }
}

impl Renderer for DebuggerRenderer {
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

        self.update_scroll(ppu.get_scroll());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn draw_fps_btn(d: &mut RaylibDrawHandle, center_x: i32, controls_y: i32, text: &str) {
    let w = 118;
    let h = 26;
    let x = center_x - w / 2;
    let y = controls_y - 20;
    d.draw_rectangle(x, y, w, h, BACKGROUND);
    d.draw_rectangle_lines(x, y, w, h, PRIMARY);
    let tw = d.measure_text(text, 12);
    d.draw_text(text, x + (w - tw) / 2, y + (h - 15) / 2, 12, PRIMARY);
}

fn draw_pad_btn(
    d: &mut RaylibDrawHandle,
    cx: i32,
    cy: i32,
    size: i32,
    rot: f32,
    key: &str,
    pressed: bool,
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    let sf = size as f32;
    let pts = [(-sf, -sf), (sf, -sf), (sf, 0.0), (0.0, sf), (-sf, 0.0)];
    let rad = rot.to_radians();
    let (s, c) = rad.sin_cos();
    let v: Vec<Vector2> = pts
        .iter()
        .map(|&(x, y)| Vector2::new(cx as f32 + x * c - y * s, cy as f32 + x * s + y * c))
        .collect();

    d.draw_triangle(v[0], v[3], v[1], bg);
    d.draw_triangle(v[0], v[4], v[3], bg);
    d.draw_triangle(v[1], v[3], v[2], bg);
    for i in 0..5 {
        d.draw_line_v(v[i], v[(i + 1) % 5], border);
    }

    if !is_mobile {
        let fs = 11;
        let tw = d.measure_text(key, fs);
        d.draw_text(key, cx - tw / 2, cy - fs / 2, fs, fg);
    }
}

fn draw_action_btn(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    size: i32,
    label: &str,
    key: &str,
    pressed: bool,
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    d.draw_rectangle(x, y, size, size, bg);
    d.draw_rectangle_lines(x, y, size, size, border);
    let fs = if is_mobile { 28 } else { 18 };
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (size - tw) / 2, y + (size - fs) / 2, fs, fg);
    if !is_mobile {
        let kfs = 8;
        let kw = d.measure_text(key, kfs);
        d.draw_text(key, x + (size - kw) / 2, y + size + 4, kfs, SECONDARY);
    }
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
    is_mobile: bool,
) {
    let (bg, fg, border) = if pressed {
        (PRIMARY, BACKGROUND, PRIMARY)
    } else {
        (BACKGROUND, FOREGROUND, SECONDARY)
    };
    d.draw_rectangle(x, y, w, h, bg);
    d.draw_rectangle_lines(x, y, w, h, border);
    let fs = if is_mobile { 14 } else { 9 };
    let tw = d.measure_text(label, fs);
    d.draw_text(label, x + (w - tw) / 2, y + (h - fs) / 2, fs, fg);
    if !is_mobile {
        d.draw_text(key, x, y + h + 3, 7, SECONDARY);
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
