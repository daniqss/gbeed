use gbeed_core::prelude::*;
use gbeed_raylib_common::input::{InputMouseTriggers, MouseButtonArea};

pub const PANEL_PADDING: i32 = 16;
pub const HEADER_HEIGHT: i32 = 34;

#[derive(Default, Debug, Clone, Copy)]
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
            let game_x = PANEL_PADDING;
            let scaled_screen_width = screen_width - PANEL_PADDING * 2;
            let scaled_screen_height =
                scaled_screen_width * DMG_SCREEN_HEIGHT as i32 / DMG_SCREEN_WIDTH as i32;
            let screen_center_x = screen_width / 2;

            let controls_y = game_y + scaled_screen_height + PANEL_PADDING * 2;

            let start_select_width = 88;
            let start_select_gap = 20;
            let start_select_total = start_select_width * 2 + start_select_gap;
            let start_select_x = screen_center_x - start_select_total / 2;
            let start_select_y = controls_y;

            let dpad_arm = 68;
            let dpad_size = 44;
            let dpad_x = screen_width / 4;
            let dpad_y = start_select_y + 160;

            let action_button_size = 76;
            let ab_gap = 28;
            let action_buttons_x = screen_width * 3 / 4;
            let action_buttons_y = dpad_y - (action_button_size + ab_gap) / 2;

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

    pub fn get_mouse_triggers(&self) -> InputMouseTriggers {
        InputMouseTriggers {
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
                self.action_buttons_y + self.action_button_size + 28,
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
