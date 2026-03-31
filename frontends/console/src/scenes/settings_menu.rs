use crate::controller::ConsoleController;
use crate::scenes::{EmulatorState, GameMenuState, SelectionMenuState};
use crate::utils::layout::{self, *};
use crate::ROMS_DIR;
use gbeed_raylib_common::{InputManager, Palette};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOption {
    ColorPalette,
}

impl SettingsOption {
    pub const ALL: [SettingsOption; 1] = [SettingsOption::ColorPalette];

    pub fn name(&self) -> &str {
        match self {
            SettingsOption::ColorPalette => "COLOR PALETTE",
        }
    }
}

pub struct SettingsMenuState {
    pub input: InputManager,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::with_debounce(0.13),
            selected: 0,
            scroll_offset: 0,
        }
    }

    pub fn update(&mut self, dt: f32, controller: &mut ConsoleController) -> Option<EmulatorState> {
        self.input.update(&controller.rl, dt);

        let count = SettingsOption::ALL.len();
        let visible_count = ((VISIBLE_BOTTOM - VISIBLE_TOP) / ITEM_H) as usize;

        if self.input.is_pressed_up() {
            if self.selected == 0 {
                self.selected = count - 1;
            } else {
                self.selected -= 1;
            }
        }

        if self.input.is_pressed_down() {
            self.selected = (self.selected + 1) % count;
        }

        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        if self.selected >= self.scroll_offset + visible_count {
            self.scroll_offset = self.selected + 1 - visible_count;
        }

        if self.input.is_repeated_left(dt) {
            return Some(EmulatorState::GameMenu(GameMenuState::new()));
        }

        if self.input.is_repeated_right(dt) {
            return Some(EmulatorState::SelectionMenu(SelectionMenuState::new(ROMS_DIR)));
        }

        let current_option = SettingsOption::ALL[self.selected];
        match current_option {
            SettingsOption::ColorPalette => {
                if self.input.is_pressed_a() {
                    let idx = Palette::ALL
                        .iter()
                        .position(|&p| p == controller.palette)
                        .unwrap_or(0);
                    controller.palette = Palette::ALL[(idx + 1) % Palette::ALL.len()];
                }
                if self.input.is_pressed_b() {
                    let idx = Palette::ALL
                        .iter()
                        .position(|&p| p == controller.palette)
                        .unwrap_or(0);
                    controller.palette =
                        Palette::ALL[if idx == 0 { Palette::ALL.len() - 1 } else { idx - 1 }];
                }
            }
        }

        None
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, palette: Palette) {
        let items: Vec<(&str, &str)> = SettingsOption::ALL
            .iter()
            .map(|opt| {
                let value: &str = match opt {
                    SettingsOption::ColorPalette => palette.name(),
                };
                (opt.name(), value)
            })
            .collect();

        layout::draw_menu_list(d, &items, self.selected, self.scroll_offset, palette);
    }
}
