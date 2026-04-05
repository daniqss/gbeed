use gbeed_core::Dmg;
use gbeed_raylib_common::{color, input::InputManager};
use raylib::prelude::*;
use std::path::PathBuf;

use crate::{
    scenes::{EmulationState, EmulatorState, GameMenuState, SettingsMenuState},
    utils::{layout::*, roms, truncate_name},
};

#[derive(Debug)]
pub struct SelectionMenuState {
    pub roms: Vec<PathBuf>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub input: InputManager,
    pub confirming_new_game: bool,
}

impl SelectionMenuState {
    pub fn new() -> Self {
        Self {
            roms: roms::find_roms(),
            selected: 0,
            scroll_offset: 0,
            input: InputManager::default(),
            confirming_new_game: false,
        }
    }

    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        dt: f32,
        rom_path: &mut Option<PathBuf>,
        gb: &mut Option<Dmg>,
        save_path: &mut Option<PathBuf>,
    ) -> Result<Option<EmulatorState>, Box<dyn std::error::Error>> {
        self.input.update(rl, dt);

        if self.confirming_new_game {
            if self.input.is_pressed_a() || self.input.is_pressed_start() {
                self.confirming_new_game = false;
                let path = self.roms[self.selected].clone();
                let cartridge = roms::load_cartridge(&path, save_path)?;
                *rom_path = Some(path);
                *gb = Some(Dmg::new(cartridge, None));

                return Ok(Some(EmulatorState::Emulation(EmulationState::new())));
            } else if self.input.is_pressed_b() {
                self.confirming_new_game = false;
            }
            return Ok(None);
        }

        let move_up = self.input.is_repeated_up(dt);
        let move_down = self.input.is_repeated_down(dt);

        let visible_count = ((VISIBLE_TOP - VISIBLE_BOTTOM) / ITEM_H) as usize;

        if !self.roms.is_empty() {
            if move_up && self.selected > 0 {
                self.selected -= 1;
            }
            if move_down && self.selected + 1 < self.roms.len() {
                self.selected += 1;
            }

            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
            if self.selected >= self.scroll_offset + visible_count {
                self.scroll_offset = self.selected + 1 - visible_count;
            }
        }

        if self.input.is_pressed_a() && !self.roms.is_empty() {
            if gb.is_some() {
                self.confirming_new_game = true;
                return Ok(None);
            }

            let path = self.roms[self.selected].clone();
            let cartridge = roms::load_cartridge(&path, save_path)?;
            *rom_path = Some(path);
            *gb = Some(Dmg::new(cartridge, None));

            return Ok(Some(EmulatorState::Emulation(EmulationState::new())));
        }

        if self.input.is_pressed_right() && !self.confirming_new_game {
            return Ok(Some(EmulatorState::GameMenu(GameMenuState::new())));
        }
        if self.input.is_repeated_left(dt) {
            return Ok(Some(EmulatorState::SettingsMenu(SettingsMenuState::new())));
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, palette_color: &color::PaletteColor) {
        if self.roms.is_empty() {
            d.draw_text(
                "no roms found",
                PADDING_X,
                VISIBLE_TOP + SECTION_PAD,
                FONT_SIZE,
                color::primary(palette_color),
            );
            return;
        }

        let text_area_w = SCROLLBAR_X - PADDING_X - 4;
        let max_chars = (text_area_w / (FONT_SIZE / 2).max(1)) as usize;

        let names: Vec<String> = self
            .roms
            .iter()
            .map(|path| {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("MissingNo");
                truncate_name(name, max_chars)
            })
            .collect();

        let items: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), "")).collect();

        draw_menu_list(d, &items, self.selected, self.scroll_offset, palette_color);

        if self.confirming_new_game {
            d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 180));

            let texts = [
                "A game is already running.",
                "Are you sure you want to",
                "load a new one?",
                "A/Start: Yes    B: No",
            ];

            let texts_widths = texts.map(|text| d.measure_text(text, 10));

            texts
                .iter()
                .zip(texts_widths.iter())
                .enumerate()
                .for_each(|(i, (text, text_width))| {
                    let color = match i {
                        0 => color::foreground(palette_color),
                        3 => color::secondary(palette_color),
                        _ => color::primary(palette_color),
                    };
                    d.draw_text(
                        text,
                        (SCREEN_WIDTH - text_width) / 2,
                        (SCREEN_HEIGHT / 2 - 20) + (i as i32) * 15,
                        10,
                        color,
                    );
                });
        }
    }
}
