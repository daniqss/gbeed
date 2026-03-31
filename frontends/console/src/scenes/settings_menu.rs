use crate::controller::ConsoleController;
use crate::scenes::{EmulatorState, GameMenuState};
use crate::utils::layout::*;
use gbeed_raylib_common::{InputManager, Palette};
use raylib::prelude::*;

pub struct SettingsMenuState {
    pub input: InputManager,
    pub selected: usize,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::with_debounce(0.13),
            selected: 0,
        }
    }

    pub fn update(&mut self, dt: f32, controller: &mut ConsoleController) -> Option<EmulatorState> {
        self.input.update(&controller.rl, dt);

        if self.input.is_pressed_left() && self.selected == 0 {
            let mut idx = Palette::ALL
                .iter()
                .position(|&p| p == controller.palette)
                .unwrap_or(0);
            if idx == 0 {
                idx = Palette::ALL.len() - 1;
            } else {
                idx -= 1;
            }
            controller.palette = Palette::ALL[idx];
        }

        if self.input.is_pressed_right() && self.selected == 0 {
            let mut idx = Palette::ALL
                .iter()
                .position(|&p| p == controller.palette)
                .unwrap_or(0);
            idx = (idx + 1) % Palette::ALL.len();
            controller.palette = Palette::ALL[idx];
        }

        // Use Escape or B to go back since Left/Right are used for settings
        if self.input.is_pressed_escape() || self.input.is_pressed_b() {
            return Some(EmulatorState::GameMenu(GameMenuState::new()));
        }

        None
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, palette: Palette) {
        let top = selector_top();
        let x = PADDING_X + 20;
        let mut y = top + 20;
        //
        {
            let title = "COLOR PALETTE";
            let title_color = if self.selected == 0 {
                gbeed_raylib_common::PRIMARY
            } else {
                gbeed_raylib_common::SECONDARY
            };
            d.draw_text(title, x, y, 10, title_color);
            y += 15;

            let value = palette.name();
            let label = format!("< {} >", value);
            let label_color = if self.selected == 0 {
                gbeed_raylib_common::FOREGROUND
            } else {
                gbeed_raylib_common::PRIMARY
            };
            d.draw_text(&label, x, y, 12, label_color);

            // Preview colors
            let colors = palette.colors();
            let preview_x = x + 140;
            for (i, c) in colors.iter().enumerate() {
                d.draw_rectangle(preview_x + (i as i32 * 14), y, 12, 12, *c);
                d.draw_rectangle_lines(
                    preview_x + (i as i32 * 14),
                    y,
                    12,
                    12,
                    gbeed_raylib_common::SECONDARY,
                );
            }
        }
    }
}
