use std::fmt::Debug;

use crate::controller::ConsoleController;
use crate::scenes::{EmulationState, EmulatorState, GameMenuState, SelectionMenuState};
use crate::utils::layout::{self, *};
use gbeed_core::Dmg;
use gbeed_raylib_common::{
    color::{Palette, PaletteColor},
    impl_cyclic_enum,
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
    DrawDebugInfo,
    Exit,
}

use SettingsOption::*;

impl_cyclic_enum!(
    SettingsOption,
    [
        ColorPalette,
        SpeedUpMode,
        SpeedUpMultiplier,
        TargetedFps,
        DrawDebugInfo,
        Exit
    ]
);

impl SettingsOption {
    pub fn name(&self) -> &str {
        match self {
            ColorPalette => "Color Palette",
            SpeedUpMode => "Speed Up Mode",
            SpeedUpMultiplier => "Speed Up Multiplier",
            TargetedFps => "Targeted FPS",
            DrawDebugInfo => "Draw Debug Info",
            Exit => "Exit",
        }
    }
}

#[derive(Debug)]
pub struct SettingsMenuState {
    pub input: InputManager,
    pub selected: SettingsOption,
    pub scroll_offset: usize,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
            selected: ColorPalette,
            scroll_offset: 0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gb: Option<&Dmg>,
        controller: &mut ConsoleController,
    ) -> Option<EmulatorState> {
        self.input.update(controller.rl, dt);

        let visible_count = ((VISIBLE_BOTTOM - VISIBLE_TOP) / ITEM_H) as usize;

        if self.input.is_pressed_up() {
            self.selected = self.selected.prev();
        }

        if self.input.is_pressed_down() {
            self.selected = self.selected.next();
        }

        let selected_idx = SettingsOption::position(&self.selected);

        if selected_idx < self.scroll_offset {
            self.scroll_offset = selected_idx;
        }
        if selected_idx >= self.scroll_offset + visible_count {
            self.scroll_offset = selected_idx + 1 - visible_count;
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

        match self.selected {
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
                    controller.speed_up_mode = controller.speed_up_mode.next();
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

            DrawDebugInfo => {
                if self.input.is_pressed_a() || self.input.is_pressed_b() {
                    controller.draw_debug_info = !controller.draw_debug_info;
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

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        palette: &Palette,
        palette_color: &PaletteColor,
        speed_up_mode: &SpeedUpMode,
        speed_up_multiplier: &SpeedUpMultiplier,
        targeted_fps: &TargetedFps,
        draw_debug_info: bool,
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
                    DrawDebugInfo if draw_debug_info => "On",
                    DrawDebugInfo => "Off",

                    Exit => "",
                };
                (opt.name(), value)
            })
            .collect();

        let selected_idx = SettingsOption::position(&self.selected);

        layout::draw_menu_list(d, &items, selected_idx, self.scroll_offset, palette_color);
    }
}
