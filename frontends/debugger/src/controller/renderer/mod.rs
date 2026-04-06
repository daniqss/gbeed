mod colors;

use gbeed_core::prelude::*;
use gbeed_core::Renderer;
use gbeed_raylib_common::{input, Texture};
use raylib::prelude::*;

use colors::GB_PALETTE;

#[allow(unused_imports)]
pub use colors::{BACKGROUND, FOREGROUND, PRIMARY, SECONDARY};

pub const PANEL_PADDING: i32 = 16;
pub const HEADER_HEIGHT: i32 = 34;

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
                is_mobile, game_x, game_y, scaled_screen_width, scaled_screen_height,
                screen_center_x, controls_y,
                middle_panel_x: 0, right_panel_x: 0, bg_map_width: 0, bg_map_height: 0,
                dpad_x, dpad_y, dpad_arm, dpad_size,
                start_select_x, start_select_y, start_select_width,
                action_buttons_x, action_buttons_y, action_button_size,
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
                is_mobile, game_x, game_y, scaled_screen_width, scaled_screen_height,
                screen_center_x, controls_y,
                middle_panel_x, right_panel_x, bg_map_width, bg_map_height,
                dpad_x, dpad_y, dpad_arm, dpad_size,
                start_select_x, start_select_y, start_select_width,
                action_buttons_x, action_buttons_y, action_button_size,
            }
        }
    }
}

pub const TILES_PER_ROW: i32 = 16;
pub const TILES_PER_COLUMN: i32 = 8;
pub const TILE_PIXEL_SIZE: i32 = 8;
pub const TILE_DISPLAY_SCALE: i32 = 3;
pub const TILE_TEXTURE_WIDTH: i32 = TILES_PER_ROW * TILE_PIXEL_SIZE;
pub const TILE_TEXTURE_HEIGHT: i32 = TILES_PER_COLUMN * TILE_PIXEL_SIZE;
pub const TILE_DISPLAY_WIDTH: i32 = TILE_TEXTURE_WIDTH * TILE_DISPLAY_SCALE;
pub const TILE_DISPLAY_HEIGHT: i32 = TILE_TEXTURE_HEIGHT * TILE_DISPLAY_SCALE;

#[derive(Copy, Clone)]
pub enum FpsMode {
    Target60,
    Target120,
    Unlimited,
}

pub struct RaylibRenderer {
    pub rl: RaylibHandle,
    pub thread: RaylibThread,

    pub screen_texture: Texture,
    pub bg_map_texture: Texture,
    pub tile_textures: [Texture; 3],

    pub buttons: input::InputState,
    pub game_name: String,
    pub game_region: String,
    pub fps_mode: FpsMode,

    pub scroll_x: i32,
    pub scroll_y: i32,
    pub layout: Layout,
}

impl RaylibRenderer {
    pub fn new(mut rl: RaylibHandle, thread: RaylibThread, layout: Layout) -> Self {
        let screen_texture = Texture::new(
            &mut rl,
            &thread,
            DMG_SCREEN_WIDTH as i32,
            DMG_SCREEN_HEIGHT as i32,
        );
        let bg_map_texture = Texture::new(&mut rl, &thread, 256, 256);
        let tile_textures = [
            Texture::new(&mut rl, &thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
            Texture::new(&mut rl, &thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
            Texture::new(&mut rl, &thread, TILE_TEXTURE_WIDTH, TILE_TEXTURE_HEIGHT),
        ];

        Self {
            rl,
            thread,
            screen_texture,
            bg_map_texture,
            tile_textures,
            buttons: input::InputState::default(),
            game_name: "Unknown".into(),
            game_region: "Unknown".into(),
            fps_mode: FpsMode::Target60,
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

    pub fn update_scroll(&mut self, x: i32, y: i32) {
        self.scroll_x = x;
        self.scroll_y = y;
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
}

impl Renderer for RaylibRenderer {
    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;

        ((self.screen_texture[index] as u32) << 16)
            | ((self.screen_texture[index + 1] as u32) << 8)
            | (self.screen_texture[index + 2] as u32)
    }

    fn write_pixel(&mut self, x: usize, y: usize, palette: u8, color_id: u8) {
        let index = (y * DMG_SCREEN_WIDTH + x) * 3;
        let shade = (palette >> (color_id * 2)) & 0x03;
        let color = GB_PALETTE[shade as usize];

        self.screen_texture[index] = color.r;
        self.screen_texture[index + 1] = color.g;
        self.screen_texture[index + 2] = color.b;
    }

    fn draw_screen(&mut self) {
        self.screen_texture.update();

        let thread = &self.thread;
        let screen_texture = &self.screen_texture.texture;
        let tile_textures = [
            &self.tile_textures[0].texture,
            &self.tile_textures[1].texture,
            &self.tile_textures[2].texture,
        ];

        let buttons = &self.buttons;
        let game_name = self.game_name.clone();
        let game_region = self.game_region.clone();

        let mut draw = self.rl.begin_drawing(thread);
        let screen_height = draw.get_screen_height();
        let layout = self.layout;

        draw.clear_background(colors::BACKGROUND);

        // vertical divider spanning full height
        if !layout.is_mobile {
            draw.draw_rectangle(
                layout.middle_panel_x - PANEL_PADDING,
                0,
                1,
                screen_height,
                colors::SECONDARY,
            );
        }

        // LEFT PANEL

        let game_x = layout.game_x;

        // header vertically centred around header_center_y
        let header_center_y = PANEL_PADDING + HEADER_HEIGHT / 2;

        let title_font_size = 22;
        let title_y = header_center_y - title_font_size / 2;
        let name_width = draw.measure_text(&game_name, title_font_size);
        draw.draw_text(&game_name, game_x, title_y, title_font_size, colors::FOREGROUND);

        let region_font_size = 11;
        let region_y = header_center_y - region_font_size / 2;
        draw.draw_text(
            &game_region,
            game_x + name_width + 10,
            region_y,
            region_font_size,
            colors::SECONDARY,
        );

        let fps_font_size = 26;
        let fps_value = draw.get_fps();
        let fps_str = format!("{fps_value:3}");
        let fps_label_font_size = 11;
        let fps_label = "fps";
        let fps_number_width = draw.measure_text(&fps_str, fps_font_size);
        let fps_label_width = draw.measure_text(fps_label, fps_label_font_size);

        let fps_group_width = fps_number_width + 4 + fps_label_width;
        let fps_x = game_x + layout.scaled_screen_width - fps_group_width;
        let fps_y = header_center_y - fps_font_size / 2;
        let fps_label_y = header_center_y - fps_label_font_size / 2;
        draw.draw_text(&fps_str, fps_x, fps_y, fps_font_size, colors::FOREGROUND);
        draw.draw_text(
            fps_label,
            fps_x + fps_number_width + 4,
            fps_label_y,
            fps_label_font_size,
            colors::SECONDARY,
        );

        // gb screen starts immediately after header
        let game_y = layout.game_y;
        draw.draw_rectangle(
            game_x - 3,
            game_y - 3,
            layout.scaled_screen_width + 6,
            layout.scaled_screen_height + 6,
            colors::PRIMARY,
        );
        draw.draw_rectangle_lines(
            game_x - 3,
            game_y - 3,
            layout.scaled_screen_width + 6,
            layout.scaled_screen_height + 6,
            colors::PRIMARY,
        );
        draw.draw_texture_pro(
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

        // controls centred under the gb screen
        let _controls_y = layout.controls_y;
        let _screen_center_x = layout.screen_center_x;

        #[cfg(not(target_arch = "wasm32"))]
        draw_fps_btn(
            &mut draw,
            _screen_center_x,
            _controls_y,
            match self.fps_mode {
                FpsMode::Target60 => "TARGET  60 Hz",
                FpsMode::Target120 => "TARGET 120 Hz",
                FpsMode::Unlimited => "TARGET  UNLIM",
            },
        );

        // dpad: centre the cross on screen_center_x - 160 (leave room for a/b on the right)
        let dpad_x = layout.dpad_x;
        let dpad_y = layout.dpad_y;
        let dpad_arm = layout.dpad_arm;
        let dpad_size = layout.dpad_size;

        // apex toward center: up=0°, down=180°, left=270°, right=90°
        draw_pad_btn(
            &mut draw,
            dpad_x,
            dpad_y - dpad_arm,
            dpad_size,
            0.0,
            "W",
            buttons.up,
        );
        draw_pad_btn(
            &mut draw,
            dpad_x,
            dpad_y + dpad_arm,
            dpad_size,
            180.0,
            "S",
            buttons.down,
        );
        draw_pad_btn(
            &mut draw,
            dpad_x - dpad_arm,
            dpad_y,
            dpad_size,
            270.0,
            "A",
            buttons.left,
        );
        draw_pad_btn(
            &mut draw,
            dpad_x + dpad_arm,
            dpad_y,
            dpad_size,
            90.0,
            "D",
            buttons.right,
        );

        // start / select
        let start_select_width = layout.start_select_width;
        let start_select_x = layout.start_select_x;
        let start_select_y = layout.start_select_y;
        draw_small_btn(
            &mut draw,
            start_select_x,
            start_select_y,
            start_select_width,
            20,
            "SELECT",
            "SHIFT / ;",
            buttons.select,
        );
        draw_small_btn(
            &mut draw,
            start_select_x + start_select_width + 18,
            start_select_y,
            start_select_width,
            20,
            "START",
            "L",
            buttons.start,
        );

        // a/b
        let action_buttons_x = layout.action_buttons_x;
        let action_buttons_y = layout.action_buttons_y;
        let action_button_size = layout.action_button_size;
        draw_action_btn(
            &mut draw,
            action_buttons_x - action_button_size / 2 + 24,
            action_buttons_y,
            action_button_size,
            "A",
            "Z / J",
            buttons.a,
        );
        draw_action_btn(
            &mut draw,
            action_buttons_x - action_button_size / 2 - 24,
            action_buttons_y + action_button_size + 8,
            action_button_size,
            "B",
            "X / K",
            buttons.b,
        );

        // Debugging panels
        if !layout.is_mobile {
            // MIDDLE PANEL (BG Map)
            let map_x = layout.middle_panel_x;
        let bg_map_y = game_y;
        let bg_map_label_y = bg_map_y - 12;
        draw.draw_text("bg map $9800", map_x, bg_map_label_y, 14, colors::SECONDARY);
        draw.draw_texture_pro(
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

        // scroll over the bg map
        let scale = layout.bg_map_width as f32 / 256.0;
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;
        let scroll_end_x = (scroll_x + 160) % 256;
        let scroll_end_y = (scroll_y + 144) % 256;
        let scroll_wraps_x = (scroll_x + 160) >= 256;
        let scroll_wraps_y = (scroll_y + 144) >= 256;

        let to_screen = |pixel_x: i32, pixel_y: i32| {
            Vector2::new(
                map_x as f32 + pixel_x as f32 * scale,
                bg_map_y as f32 + pixel_y as f32 * scale,
            )
        };

        const OVERLAY_COLOR: Color = Color {
            r: 255,
            g: 220,
            b: 0,
            a: 255,
        };
        const OVERLAY_THICKNESS: f32 = 1.5;

        for &row_y in &[scroll_y, scroll_end_y] {
            if !scroll_wraps_x {
                draw.draw_line_ex(
                    to_screen(scroll_x, row_y),
                    to_screen(scroll_x + 160, row_y),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
            } else {
                draw.draw_line_ex(
                    to_screen(scroll_x, row_y),
                    to_screen(256, row_y),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
                draw.draw_line_ex(
                    to_screen(0, row_y),
                    to_screen(scroll_end_x, row_y),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
            }
        }
        for &column_x in &[scroll_x, scroll_end_x] {
            if !scroll_wraps_y {
                draw.draw_line_ex(
                    to_screen(column_x, scroll_y),
                    to_screen(column_x, scroll_y + 144),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
            } else {
                draw.draw_line_ex(
                    to_screen(column_x, scroll_y),
                    to_screen(column_x, 256),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
                draw.draw_line_ex(
                    to_screen(column_x, 0),
                    to_screen(column_x, scroll_end_y),
                    OVERLAY_THICKNESS,
                    OVERLAY_COLOR,
                );
            }
        }

        // RIGHT PANEL (Tiles)
        let right_panel_x = layout.right_panel_x;

        const TV_LABELS: [&str; 3] = [
            "vram  $8000-$87ff  (block 0)",
            "vram  $8800-$8fff  (block 1)",
            "vram  $9000-$97ff  (block 2)",
        ];
        let tv_stride = TILE_DISPLAY_HEIGHT + 16;

        for i in 0..3_usize {
            let tile_texture_y = game_y + i as i32 * tv_stride;
            let tile_label_y = tile_texture_y - 12;

            draw.draw_text(TV_LABELS[i], right_panel_x, tile_label_y, 14, colors::SECONDARY);
            draw.draw_texture_pro(
                &tile_textures[i],
                Rectangle::new(0.0, 0.0, TILE_TEXTURE_WIDTH as f32, TILE_TEXTURE_HEIGHT as f32),
                Rectangle::new(
                    right_panel_x as f32,
                    tile_texture_y as f32,
                    TILE_DISPLAY_WIDTH as f32,
                    TILE_DISPLAY_HEIGHT as f32,
                ),
                Vector2::ZERO,
                0.0,
                Color::WHITE,
            );

            let mut grid_color = colors::BACKGROUND;
            grid_color.a = 60;
            let grid_cell_size = TILE_PIXEL_SIZE * TILE_DISPLAY_SCALE;
            for column in 0..=TILES_PER_ROW {
                draw.draw_line(
                    right_panel_x + column * grid_cell_size,
                    tile_texture_y,
                    right_panel_x + column * grid_cell_size,
                    tile_texture_y + TILE_DISPLAY_HEIGHT,
                    grid_color,
                );
            }
            for row in 0..=TILES_PER_COLUMN {
                draw.draw_line(
                    right_panel_x,
                    tile_texture_y + row * grid_cell_size,
                    right_panel_x + TILE_DISPLAY_WIDTH,
                    tile_texture_y + row * grid_cell_size,
                    grid_color,
                );
            }
        }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn draw_fps_btn(draw: &mut RaylibDrawHandle, screen_center_x: i32, controls_y: i32, target_str: &str) {
    // fps button above others
    let fps_button_width = 118_i32;
    let fps_button_height = 26_i32;
    let fps_button_x = screen_center_x - fps_button_width / 2;
    let fps_button_y = controls_y - 20;

    draw.draw_rectangle(
        fps_button_x,
        fps_button_y,
        fps_button_width,
        fps_button_height,
        colors::BACKGROUND,
    );
    draw.draw_rectangle_lines(
        fps_button_x,
        fps_button_y,
        fps_button_width,
        fps_button_height,
        colors::PRIMARY,
    );
    let text_width = draw.measure_text(target_str, 12);
    draw.draw_text(
        target_str,
        fps_button_x + (fps_button_width - text_width) / 2,
        fps_button_y + (fps_button_height - 15) / 2,
        12,
        colors::PRIMARY,
    );
}

fn draw_pad_btn(
    draw: &mut RaylibDrawHandle,
    center_x: i32,
    center_y: i32,
    size: i32,
    rotation_deg: f32,
    key: &str,
    pressed: bool,
) {
    let (background_color, foreground_color, border_color) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };

    let size_f = size as f32;
    let base_points: [(f32, f32); 5] = [
        (-size_f, -size_f),
        (size_f, -size_f),
        (size_f, 0.0),
        (0.0, size_f),
        (-size_f, 0.0),
    ];

    let rotation_rad = rotation_deg.to_radians();
    let (sin_r, cos_r) = rotation_rad.sin_cos();

    let vertices: Vec<Vector2> = base_points
        .iter()
        .map(|&(x, y)| {
            Vector2::new(
                center_x as f32 + x * cos_r - y * sin_r,
                center_y as f32 + x * sin_r + y * cos_r,
            )
        })
        .collect();

    // fan of 3 triangles, CCW winding in screen space
    draw.draw_triangle(vertices[0], vertices[3], vertices[1], background_color); // A D B
    draw.draw_triangle(vertices[0], vertices[4], vertices[3], background_color); // A E D
    draw.draw_triangle(vertices[1], vertices[3], vertices[2], background_color); // B D C

    // outline
    for i in 0..5 {
        draw.draw_line_v(vertices[i], vertices[(i + 1) % 5], border_color);
    }

    let font_size = 11;
    let text_width = draw.measure_text(key, font_size);
    draw.draw_text(
        key,
        center_x - text_width / 2,
        center_y - font_size / 2,
        font_size,
        foreground_color,
    );
}

fn draw_action_btn(
    draw: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    size: i32,
    label: &str,
    key: &str,
    pressed: bool,
) {
    let (background_color, foreground_color, border_color) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };

    draw.draw_rectangle(x, y, size, size, background_color);
    draw.draw_rectangle_lines(x, y, size, size, border_color);

    let font_size = 18;
    let text_width = draw.measure_text(label, font_size);
    draw.draw_text(
        label,
        x + (size - text_width) / 2,
        y + (size - font_size) / 2,
        font_size,
        foreground_color,
    );

    let key_font_size = 8;
    let key_width = draw.measure_text(key, key_font_size);
    draw.draw_text(
        key,
        x + (size - key_width) / 2,
        y + size + 4,
        key_font_size,
        colors::SECONDARY,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_small_btn(
    draw: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    label: &str,
    key: &str,
    pressed: bool,
) {
    let (background_color, foreground_color, border_color) = if pressed {
        (colors::PRIMARY, colors::BACKGROUND, colors::PRIMARY)
    } else {
        (colors::BACKGROUND, colors::FOREGROUND, colors::SECONDARY)
    };

    draw.draw_rectangle(x, y, width, height, background_color);
    draw.draw_rectangle_lines(x, y, width, height, border_color);

    let font_size = 9;
    let text_width = draw.measure_text(label, font_size);
    draw.draw_text(
        label,
        x + (width - text_width) / 2,
        y + (height - font_size) / 2,
        font_size,
        foreground_color,
    );
    draw.draw_text(key, x, y + height + 3, 7, colors::SECONDARY);
}

pub fn update_bg_map(
    texture: &mut Texture,
    map_data: &[u8],
    tile_data: &[u8],
    is_mode_8000: bool,
    palette: u8,
) {
    let stride = 256_usize;

    for tile_y in 0..32_usize {
        for tile_x in 0..32_usize {
            let tile_number = map_data[tile_y * 32 + tile_x];

            let tile_base = if is_mode_8000 {
                (tile_number as usize) * 16
            } else {
                let signed_tile_number = tile_number as i8 as i32;
                (0x1000_i32 + signed_tile_number * 16) as usize
            };

            let pixel_x_base = tile_x * 8;
            let pixel_y_base = tile_y * 8;

            for row in 0..8_usize {
                let low_byte = tile_data[tile_base + row * 2];
                let high_byte = tile_data[tile_base + row * 2 + 1];

                for column in 0..8_usize {
                    let bit_index = 7 - column;
                    let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);

                    let shade = (palette >> (color_id * 2)) & 0x03;
                    let color = colors::GB_PALETTE[shade as usize];
                    let index = ((pixel_y_base + row) * stride + (pixel_x_base + column)) * 3;

                    texture[index] = color.r;
                    texture[index + 1] = color.g;
                    texture[index + 2] = color.b;
                }
            }
        }
    }

    texture.update();
}

// decodes a 2bpp vram block into the tile texture (region 0/$8000, 1/$8800, 2/$9000)
pub fn update_tiles(texture: &mut Texture, data: &[u8]) {
    let stride = TILE_TEXTURE_WIDTH as usize;

    for tile_index in 0..128_usize {
        let tile_base_x = (tile_index % TILES_PER_ROW as usize) * TILE_PIXEL_SIZE as usize;
        let tile_base_y = (tile_index / TILES_PER_ROW as usize) * TILE_PIXEL_SIZE as usize;
        let data_base = tile_index * 16;

        for row in 0..8_usize {
            let low_byte = data[data_base + row * 2];
            let high_byte = data[data_base + row * 2 + 1];

            for column in 0..8_usize {
                let bit_index = 7 - column;
                let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);

                let color = colors::GB_PALETTE[color_id as usize];
                let pixel_x = tile_base_x + column;
                let pixel_y = tile_base_y + row;
                let index = (pixel_y * stride + pixel_x) * 3;

                texture[index] = color.r;
                texture[index + 1] = color.g;
                texture[index + 2] = color.b;
            }
        }
    }

    texture.update();
}
