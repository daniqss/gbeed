use crate::controller::{
    DebuggerController, TILE_DISPLAY_HEIGHT, TILE_DISPLAY_SCALE, TILE_DISPLAY_WIDTH, TILE_PIXEL_SIZE,
    TILE_TEXTURE_HEIGHT, TILE_TEXTURE_WIDTH, TILES_PER_COLUMN, TILES_PER_ROW,
};
use crate::scenes::EmulatorState;
use crate::utils::{
    BACKGROUND, FOREGROUND, HEADER_HEIGHT, Layout, PANEL_PADDING, PRIMARY, SECONDARY, components::*,
};
use gbeed_core::prelude::*;
use gbeed_raylib_common::Texture;
use gbeed_raylib_common::input::InputManager;
use raylib::prelude::*;

#[derive(Default, Debug)]
pub struct EmulationScene {
    pub layout: Layout,
    pub input: InputManager,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub game_name: String,
    pub game_region: String,
}

impl EmulationScene {
    pub fn new(layout: Layout, game_name: String, game_region: String) -> Self {
        Self {
            input: InputManager::new(0.08, None, Some(layout.get_mouse_triggers()), None),
            layout,
            scroll_x: 0,
            scroll_y: 0,
            game_name,
            game_region,
        }
    }

    pub fn update_layout(&mut self, layout: Layout) {
        self.layout = layout;
        self.input.mouse_triggers = Some(layout.get_mouse_triggers());
    }

    pub fn update(
        &mut self,
        dt: f32,
        gb: &mut Option<Dmg>,
        controller: &mut DebuggerController,
    ) -> Result<Option<EmulatorState>, Box<dyn std::error::Error>> {
        self.input.update(&controller.rl, dt);

        if self.input.is_held_escape() {
            return Ok(Some(EmulatorState::Exit));
        }

        if let Some(gb) = gb {
            self.input.state().apply(&mut gb.joypad);
            gb.run(controller)?;
        }

        Ok(None)
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        screen_texture: &Texture,
        tile_textures: &[Texture; 3],
        bg_map_texture: &Texture,
    ) {
        if !self.layout.is_mobile {
            self.draw_separators(d);
        }

        self.draw_header(d);
        self.draw_screen(d, screen_texture);
        self.draw_controls(d);

        if !self.layout.is_mobile {
            self.draw_debug_panels(d, tile_textures, bg_map_texture);
        }
    }

    fn draw_separators(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle(
            self.layout.middle_panel_x - PANEL_PADDING,
            0,
            1,
            d.get_screen_height(),
            SECONDARY,
        );
    }

    fn draw_header(&self, d: &mut RaylibDrawHandle) {
        let header_y = PANEL_PADDING + HEADER_HEIGHT / 2;

        // Game info
        let title_font_size = 22;
        let name_width = d.measure_text(&self.game_name, title_font_size);
        d.draw_text(
            &self.game_name,
            self.layout.game_x,
            header_y - title_font_size / 2,
            title_font_size,
            FOREGROUND,
        );

        let region_font_size = 11;
        d.draw_text(
            &self.game_region,
            self.layout.game_x + name_width + 10,
            header_y - region_font_size / 2,
            region_font_size,
            SECONDARY,
        );

        // FPS
        let fps_font_size = 26;
        let fps_str = format!("{:3}", d.get_fps());
        let fps_width = d.measure_text(&fps_str, fps_font_size);
        let fps_x = self.layout.game_x + self.layout.scaled_screen_width - (fps_width + 30);

        d.draw_text(
            &fps_str,
            fps_x,
            header_y - fps_font_size / 2,
            fps_font_size,
            FOREGROUND,
        );
        d.draw_text("fps", fps_x + fps_width + 4, header_y - 5, 11, SECONDARY);
    }

    fn draw_screen(&self, d: &mut RaylibDrawHandle, texture: &Texture) {
        let (x, y) = (self.layout.game_x, self.layout.game_y);
        let (w, h) = (self.layout.scaled_screen_width, self.layout.scaled_screen_height);

        // Border
        d.draw_rectangle(x - 3, y - 3, w + 6, h + 6, PRIMARY);

        // Texture
        d.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(x as f32, y as f32, w as f32, h as f32),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );
    }

    fn draw_controls(&self, d: &mut RaylibDrawHandle) {
        self.draw_dpad(d);
        self.draw_system_buttons(d);
        self.draw_action_buttons(d);
    }

    fn draw_dpad(&self, d: &mut RaylibDrawHandle) {
        let (x, y) = (self.layout.dpad_x, self.layout.dpad_y);
        let arm = self.layout.dpad_arm;
        let size = self.layout.dpad_size;
        let is_mobile = self.layout.is_mobile;

        draw_pad_btn(d, x, y - arm, size, 0.0, "W", self.input.is_held_up(), is_mobile);
        draw_pad_btn(
            d,
            x,
            y + arm,
            size,
            180.0,
            "S",
            self.input.is_held_down(),
            is_mobile,
        );
        draw_pad_btn(
            d,
            x - arm,
            y,
            size,
            270.0,
            "A",
            self.input.is_held_left(),
            is_mobile,
        );
        draw_pad_btn(
            d,
            x + arm,
            y,
            size,
            90.0,
            "D",
            self.input.is_held_right(),
            is_mobile,
        );
    }

    fn draw_system_buttons(&self, d: &mut RaylibDrawHandle) {
        let (x, y) = (self.layout.start_select_x, self.layout.start_select_y);
        let w = self.layout.start_select_width;
        let is_mobile = self.layout.is_mobile;

        draw_small_btn(
            d,
            x,
            y,
            w,
            20,
            "SELECT",
            "SHIFT / ;",
            self.input.is_held_select(),
            is_mobile,
        );
        draw_small_btn(
            d,
            x + w + 18,
            y,
            w,
            20,
            "START",
            "L",
            self.input.is_held_start(),
            is_mobile,
        );
    }

    fn draw_action_buttons(&self, d: &mut RaylibDrawHandle) {
        let (x, y) = (self.layout.action_buttons_x, self.layout.action_buttons_y);
        let size = self.layout.action_button_size;
        let is_mobile = self.layout.is_mobile;

        draw_action_btn(
            d,
            x - size / 2 + 24,
            y,
            size,
            "A",
            "Z / J",
            self.input.is_held_a(),
            is_mobile,
        );
        draw_action_btn(
            d,
            x - size / 2 - 24,
            y + size + 8,
            size,
            "B",
            "X / K",
            self.input.is_held_b(),
            is_mobile,
        );
    }

    fn draw_debug_panels(
        &self,
        d: &mut RaylibDrawHandle,
        tile_textures: &[Texture; 3],
        bg_map_texture: &Texture,
    ) {
        self.draw_bg_map_panel(d, bg_map_texture);
        self.draw_tile_panels(d, tile_textures);
    }

    fn draw_bg_map_panel(&self, d: &mut RaylibDrawHandle, texture: &Texture) {
        let (x, y) = (self.layout.middle_panel_x, self.layout.game_y);
        let (w, h) = (self.layout.bg_map_width, self.layout.bg_map_height);

        d.draw_text("bg map $9800", x, y - 12, 14, SECONDARY);
        d.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, 256.0, 256.0),
            Rectangle::new(x as f32, y as f32, w as f32, h as f32),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        self.draw_scroll_overlay(d, x, y, w as f32 / 256.0);
    }

    fn draw_scroll_overlay(&self, d: &mut RaylibDrawHandle, x: i32, y: i32, scale: f32) {
        let (sx, sy) = (self.scroll_x, self.scroll_y);
        let ex = (sx + 160) % 256;
        let ey = (sy + 144) % 256;
        let wraps_x = (sx + 160) >= 256;
        let wraps_y = (sy + 144) >= 256;

        let to_screen =
            |px: i32, py: i32| Vector2::new(x as f32 + px as f32 * scale, y as f32 + py as f32 * scale);

        const OVERLAY_COLOR: Color = Color {
            r: 255,
            g: 220,
            b: 0,
            a: 255,
        };

        // horizontal lines
        for &ry in &[sy, ey] {
            if !wraps_x {
                d.draw_line_ex(to_screen(sx, ry), to_screen(sx + 160, ry), 1.5, OVERLAY_COLOR);
            } else {
                d.draw_line_ex(to_screen(sx, ry), to_screen(256, ry), 1.5, OVERLAY_COLOR);
                d.draw_line_ex(to_screen(0, ry), to_screen(ex, ry), 1.5, OVERLAY_COLOR);
            }
        }
        // vertical lines
        for &cx in &[sx, ex] {
            if !wraps_y {
                d.draw_line_ex(to_screen(cx, sy), to_screen(cx, sy + 144), 1.5, OVERLAY_COLOR);
            } else {
                d.draw_line_ex(to_screen(cx, sy), to_screen(cx, 256), 1.5, OVERLAY_COLOR);
                d.draw_line_ex(to_screen(cx, 0), to_screen(cx, ey), 1.5, OVERLAY_COLOR);
            }
        }
    }

    fn draw_tile_panels(&self, d: &mut RaylibDrawHandle, textures: &[Texture; 3]) {
        let x = self.layout.right_panel_x;
        let stride = TILE_DISPLAY_HEIGHT + 16;
        const LABELS: [&str; 3] = [
            "vram  $8000-$87ff  (block 0)",
            "vram  $8800-$8fff  (block 1)",
            "vram  $9000-$97ff  (block 2)",
        ];

        for i in 0..3 {
            let y = self.layout.game_y + i as i32 * stride;
            d.draw_text(LABELS[i], x, y - 12, 14, SECONDARY);
            d.draw_texture_pro(
                &textures[i],
                Rectangle::new(0.0, 0.0, TILE_TEXTURE_WIDTH as f32, TILE_TEXTURE_HEIGHT as f32),
                Rectangle::new(
                    x as f32,
                    y as f32,
                    TILE_DISPLAY_WIDTH as f32,
                    TILE_DISPLAY_HEIGHT as f32,
                ),
                Vector2::ZERO,
                0.0,
                Color::WHITE,
            );
            self.draw_tile_grid(d, x, y);
        }
    }

    fn draw_tile_grid(&self, d: &mut RaylibDrawHandle, x: i32, y: i32) {
        let mut grid_color = BACKGROUND;
        grid_color.a = 60;
        let cell = TILE_PIXEL_SIZE * TILE_DISPLAY_SCALE;

        for col in 0..=TILES_PER_ROW {
            let lx = x + col * cell;
            d.draw_line(lx, y, lx, y + TILE_DISPLAY_HEIGHT, grid_color);
        }
        for row in 0..=TILES_PER_COLUMN {
            let ly = y + row * cell;
            d.draw_line(x, ly, x + TILE_DISPLAY_WIDTH, ly, grid_color);
        }
    }
}
