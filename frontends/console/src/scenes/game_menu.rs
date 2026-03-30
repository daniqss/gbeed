use crate::scenes::{EmulationState, EmulatorState, SelectionMenuState, SettingsMenuState};
use crate::utils::layout::*;
use crate::ROMS_DIR;
use gbeed_core::prelude::{Dmg, DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH};
use gbeed_raylib_common::{InputKeyTriggers, InputState, ToInputState};
use raylib::prelude::*;
use std::path::PathBuf;

pub struct GameMenuState {
    pub key_triggers: InputKeyTriggers,
    pub last_input: InputState,
    pub debounce_timer: f32,
}

impl Default for GameMenuState {
    fn default() -> Self {
        Self {
            key_triggers: InputKeyTriggers::default(),
            last_input: InputState::default(),
            debounce_timer: 0.13,
        }
    }
}

impl GameMenuState {
    pub fn update(&mut self, rl: &RaylibHandle, dt: f32, gb: &Option<Dmg>) -> Option<EmulatorState> {
        if self.debounce_timer > 0.0 {
            self.debounce_timer -= dt;
            return None;
        }

        let input = self.key_triggers.to_input(rl);

        let a_pressed = input.a && !self.last_input.a;
        let start_pressed = input.start && !self.last_input.start;
        let select_pressed = input.select && !self.last_input.select;
        let left_pressed = input.left && !self.last_input.left;
        let right_pressed = input.right && !self.last_input.right;

        self.last_input = input;

        if (a_pressed || start_pressed || select_pressed) && gb.is_some() {
            return Some(EmulatorState::Emulation(EmulationState {
                key_triggers: InputKeyTriggers::default(),
            }));
        }

        if left_pressed {
            return Some(EmulatorState::SelectionMenu(SelectionMenuState::new(ROMS_DIR)));
        }

        if right_pressed {
            return Some(EmulatorState::SettingsMenu(SettingsMenuState::default()));
        }

        None
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        screen: &gbeed_raylib_common::Texture,
        gb: &Option<Dmg>,
        rom_path: &Option<PathBuf>,
    ) {
        let info_x = PADDING_X + 10;
        let info_y = selector_top() + 10;

        let Some(gb) = gb else {
            let text = "No ROM loaded";
            let text_w = d.measure_text(text, 14);
            d.draw_text(
                text,
                (SCREEN_WIDTH - text_w) / 2,
                SCREEN_HEIGHT / 2,
                14,
                gbeed_raylib_common::PRIMARY,
            );
            return;
        };

        // draw the emulator screen in the background if a rom is loaded
        d.draw_texture_pro(
            &screen.texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        // make it opaque/darker to show it's paused
        d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 200));

        let file_name = rom_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let game_title = &gb.cartridge.header.title;
        let header_info = gb.cartridge.header.to_string_array();

        d.draw_text(
            &format!("{} - {}", file_name, game_title),
            info_x,
            info_y,
            14,
            gbeed_raylib_common::FOREGROUND,
        );

        let mut y_offset = info_y + 20;
        for line in header_info {
            d.draw_text(&line, info_x, y_offset, 10, gbeed_raylib_common::PRIMARY);
            y_offset += 14;
        }
    }
}
