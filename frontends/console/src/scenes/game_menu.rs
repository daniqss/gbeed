use crate::scenes::{EmulationState, EmulatorState, SelectionMenuState, SettingsMenuState};
use crate::utils::layout::*;
use gbeed_core::prelude::{Dmg, DMG_SCREEN_HEIGHT, DMG_SCREEN_WIDTH};
use gbeed_raylib_common::{InputManager, Palette};
use raylib::prelude::*;
use std::path::PathBuf;

pub struct GameMenuState {
    pub input: InputManager,
}

impl GameMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32, gb: &Option<Dmg>) -> Option<EmulatorState> {
        self.input.update(rl, dt);

        if (self.input.is_pressed_a() || self.input.is_pressed_start() || self.input.is_pressed_select())
            && gb.is_some()
        {
            return Some(EmulatorState::Emulation(EmulationState::new()));
        }

        if self.input.is_repeated_left(dt) {
            return Some(EmulatorState::SelectionMenu(SelectionMenuState::new()));
        }

        if self.input.is_repeated_right(dt) {
            return Some(EmulatorState::SettingsMenu(SettingsMenuState::new()));
        }

        None
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        screen: &gbeed_raylib_common::Texture,
        gb: &Option<Dmg>,
        rom_path: &Option<PathBuf>,
        palette: Palette,
    ) {
        let info_x = PADDING_X + 10;
        let info_y = VISIBLE_TOP + 10;

        let Some(gb) = gb else {
            let text = "No ROM loaded";
            let text_w = d.measure_text(text, 14);
            d.draw_text(
                text,
                (SCREEN_WIDTH - text_w) / 2,
                SCREEN_HEIGHT / 2,
                14,
                palette.primary(),
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
            palette.foreground(),
        );

        let mut y_offset = info_y + 20;
        for line in header_info {
            d.draw_text(&line, info_x, y_offset, 10, palette.primary());
            y_offset += 14;
        }
    }
}
