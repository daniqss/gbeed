use crate::controller::{ConsoleController, SpeedUpMode};
use crate::scenes::{EmulationState, EmulatorState, GameMenuState, SelectionMenuState};
use crate::utils::layout::{self, *};
use gbeed_core::Dmg;
use gbeed_raylib_common::{color, input::InputManager};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOption {
    ColorPalette,
    SpeedUpMode,
    Exit,
}

impl SettingsOption {
    pub const ALL: [SettingsOption; 3] = [
        SettingsOption::ColorPalette,
        SettingsOption::SpeedUpMode,
        SettingsOption::Exit,
    ];

    pub fn name(&self) -> &str {
        match self {
            SettingsOption::ColorPalette => "Color Palette",
            SettingsOption::SpeedUpMode => "Speed Up Mode",
            SettingsOption::Exit => "Exit",
        }
    }
}

#[derive(Debug)]
pub struct SettingsMenuState {
    pub input: InputManager,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gb: Option<&Dmg>,
        controller: &mut ConsoleController,
    ) -> Option<EmulatorState> {
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
            return Some(EmulatorState::SelectionMenu(SelectionMenuState::new()));
        }
        if self.input.is_pressed_escape() && gb.is_some() {
            return Some(EmulatorState::Emulation(EmulationState::new()));
        }

        let current_option = SettingsOption::ALL[self.selected];

        match current_option {
            SettingsOption::ColorPalette => {
                if self.input.is_pressed_a() {
                    controller.palette = controller.palette.next();
                }
                if self.input.is_pressed_b() {
                    controller.palette = controller.palette.prev();
                }
            }
            SettingsOption::SpeedUpMode => {
                if self.input.is_pressed_a() || self.input.is_pressed_b() {
                    controller.speed_up_mode = match controller.speed_up_mode {
                        SpeedUpMode::Toggle(_) => SpeedUpMode::Hold,
                        SpeedUpMode::Hold => SpeedUpMode::Toggle(false),
                    };
                }
            }
            SettingsOption::Exit => {
                if self.input.is_pressed_a() {
                    return Some(EmulatorState::Exit);
                }
            }
        }

        controller.palette_color = controller.palette.get_palette_color();

        None
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        palette: &color::Palette,
        speed_up_mode: &SpeedUpMode,
        palette_color: &color::PaletteColor,
    ) {
        let items: Vec<(&str, &str)> = SettingsOption::ALL
            .iter()
            .map(|opt| {
                let value: &str = match opt {
                    SettingsOption::ColorPalette => palette.name(),
                    SettingsOption::SpeedUpMode => match speed_up_mode {
                        SpeedUpMode::Toggle(_) => "Toggle",
                        SpeedUpMode::Hold => "Hold",
                    },
                    SettingsOption::Exit => "",
                };
                (opt.name(), value)
            })
            .collect();

        layout::draw_menu_list(d, &items, self.selected, self.scroll_offset, palette_color);
    }
}
