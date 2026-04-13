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
            layout,
            input: InputManager::default(),
            scroll_x: 0,
            scroll_y: 0,
            game_name,
            game_region,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gb: &mut Option<Dmg>,
        controller: &mut DebuggerController,
    ) -> Result<Option<EmulatorState>, Box<dyn std::error::Error>> {
        self.input.update(&controller.rl, dt);

        if self.input.is_pressed_escape() {
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
        let game_name = "umm game".to_string();
        let game_region = "???".to_string();
        if !self.layout.is_mobile {
            d.draw_rectangle(
                self.layout.middle_panel_x - PANEL_PADDING,
                0,
                1,
                d.get_screen_height(),
                SECONDARY,
            );
        }

        // Header
        let game_x = self.layout.game_x;
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
        let fps_x = game_x + self.layout.scaled_screen_width - (fps_number_width + 30);
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
        let game_y = self.layout.game_y;
        d.draw_rectangle(
            game_x - 3,
            game_y - 3,
            self.layout.scaled_screen_width + 6,
            self.layout.scaled_screen_height + 6,
            PRIMARY,
        );
        d.draw_texture_pro(
            screen_texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(
                game_x as f32,
                game_y as f32,
                self.layout.scaled_screen_width as f32,
                self.layout.scaled_screen_height as f32,
            ),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // D-PAD
        let dpad_x = self.layout.dpad_x;
        let dpad_y = self.layout.dpad_y;
        let dpad_arm = self.layout.dpad_arm;
        let dpad_size = self.layout.dpad_size;
        draw_pad_btn(
            d,
            dpad_x,
            dpad_y - dpad_arm,
            dpad_size,
            0.0,
            "W",
            self.input.is_pressed_speed_up(),
            self.layout.is_mobile,
        );
        draw_pad_btn(
            d,
            dpad_x,
            dpad_y + dpad_arm,
            dpad_size,
            180.0,
            "S",
            self.input.is_pressed_down(),
            self.layout.is_mobile,
        );
        draw_pad_btn(
            d,
            dpad_x - dpad_arm,
            dpad_y,
            dpad_size,
            270.0,
            "A",
            self.input.is_pressed_left(),
            self.layout.is_mobile,
        );
        draw_pad_btn(
            d,
            dpad_x + dpad_arm,
            dpad_y,
            dpad_size,
            90.0,
            "D",
            self.input.is_pressed_right(),
            self.layout.is_mobile,
        );

        // Start/Select
        draw_small_btn(
            d,
            self.layout.start_select_x,
            self.layout.start_select_y,
            self.layout.start_select_width,
            20,
            "SELECT",
            "SHIFT / ;",
            self.input.is_pressed_select(),
            self.layout.is_mobile,
        );
        draw_small_btn(
            d,
            self.layout.start_select_x + self.layout.start_select_width + 18,
            self.layout.start_select_y,
            self.layout.start_select_width,
            20,
            "START",
            "L",
            self.input.is_pressed_start(),
            self.layout.is_mobile,
        );

        // A/B
        let action_x = self.layout.action_buttons_x;
        let action_y = self.layout.action_buttons_y;
        let action_size = self.layout.action_button_size;
        draw_action_btn(
            d,
            action_x - action_size / 2 + 24,
            action_y,
            action_size,
            "A",
            "Z / J",
            self.input.is_pressed_a(),
            self.layout.is_mobile,
        );
        draw_action_btn(
            d,
            action_x - action_size / 2 - 24,
            action_y + action_size + 8,
            action_size,
            "B",
            "X / K",
            self.input.is_pressed_b(),
            self.layout.is_mobile,
        );

        // debug panels, only on desktop
        if !self.layout.is_mobile {
            self.draw_debug_panels(d, tile_textures, bg_map_texture);
        }
    }

    fn draw_debug_panels(
        &self,
        d: &mut RaylibDrawHandle,
        tile_textures: &[Texture; 3],
        bg_map_texture: &Texture,
    ) {
        let game_y = self.layout.game_y;
        let map_x = self.layout.middle_panel_x;
        let bg_map_y = game_y;
        let scroll_x = self.scroll_x;
        let scroll_y = self.scroll_y;

        d.draw_text("bg map $9800", map_x, bg_map_y - 12, 14, SECONDARY);
        d.draw_texture_pro(
            bg_map_texture,
            Rectangle::new(0.0, 0.0, 256.0, 256.0),
            Rectangle::new(
                map_x as f32,
                bg_map_y as f32,
                self.layout.bg_map_width as f32,
                self.layout.bg_map_height as f32,
            ),
            Vector2::ZERO,
            0.0,
            Color::WHITE,
        );

        // Scroll overlay
        let scale = self.layout.bg_map_width as f32 / 256.0;
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
        let right_x = self.layout.right_panel_x;
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
                &tile_textures[i],
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
