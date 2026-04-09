use crate::controller::ConsoleController;
use crate::scenes::{EmulationState, EmulatorState, GameMenuState, SelectionMenuState};
use crate::utils::layout::{self, *};
use gbeed_core::Dmg;
use gbeed_raylib_common::{
    color::{Palette, PaletteColor},
    input::InputManager,
    settings::{SpeedUpMode, SpeedUpMultiplier, TargetedFps},
};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsOption {
    ColorPalette,
    SpeedUpMode,
    SpeedUpMultiplier,
    TargetedFps,
    Exit,
}

use SettingsOption::*;

impl SettingsOption {
    pub const ALL: [SettingsOption; 5] = [ColorPalette, SpeedUpMode, SpeedUpMultiplier, TargetedFps, Exit];

    pub fn name(&self) -> &str {
        match self {
            ColorPalette => "Color Palette",
            SpeedUpMode => "Speed Up Mode",
            SpeedUpMultiplier => "Speed Up Multiplier",
            TargetedFps => "Targeted FPS",
            Exit => "Exit",
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
            ColorPalette => {
                if self.input.is_pressed_a() {
                    controller.palette = controller.palette.next();
                }
                if self.input.is_pressed_b() {
                    controller.palette = controller.palette.prev();
                }
            }
            SpeedUpMode => {
                if self.input.is_pressed_a() || self.input.is_pressed_b() {
                    controller.speed_up_mode = match controller.speed_up_mode {
                        SpeedUpMode::Toggle(_) => SpeedUpMode::Hold,
                        SpeedUpMode::Hold => SpeedUpMode::Toggle(false),
                    };
                }
            }

            SpeedUpMultiplier => {
                if self.input.is_pressed_a() {
                    controller.speed_up_multiplier = controller.speed_up_multiplier.next();
                }
                if self.input.is_pressed_b() {
                    controller.speed_up_multiplier = controller.speed_up_multiplier.prev();
                }
            }

            TargetedFps => {
                if self.input.is_pressed_a() {
                    controller.targeted_fps = controller.targeted_fps.next();
                    controller.rl.set_target_fps(controller.targeted_fps as u32);
                }
                if self.input.is_pressed_b() {
                    controller.targeted_fps = controller.targeted_fps.prev();
                    controller.rl.set_target_fps(controller.targeted_fps as u32);
                }
            }

            Exit => {
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
        palette: &Palette,
        palette_color: &PaletteColor,
        speed_up_mode: &SpeedUpMode,
        speed_up_multiplier: &SpeedUpMultiplier,
        targeted_fps: &TargetedFps,
    ) {
        let items: Vec<(&str, &str)> = SettingsOption::ALL
            .iter()
            .map(|opt| {
                let value: &str = match opt {
                    ColorPalette => palette.name(),
                    SpeedUpMode => match speed_up_mode {
                        SpeedUpMode::Toggle(_) => "Toggle",
                        SpeedUpMode::Hold => "Hold",
                    },
                    SpeedUpMultiplier => match speed_up_multiplier {
                        SpeedUpMultiplier::OneAndHalf => "1.5x",
                        SpeedUpMultiplier::Double => "2x",
                        SpeedUpMultiplier::Cuadruple => "4x",
                    },
                    TargetedFps => match targeted_fps {
                        TargetedFps::Target30 => "30",
                        TargetedFps::Target60 => "60",
                        TargetedFps::Unlimited => "Unlimited",
                    },

                    Exit => "",
                };
                (opt.name(), value)
            })
            .collect();

        layout::draw_menu_list(d, &items, self.selected, self.scroll_offset, palette_color);
    }
}
