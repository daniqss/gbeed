use gbeed_core::prelude::Dmg;
use gbeed_raylib_common::InputManager;
use raylib::prelude::*;
use std::path::PathBuf;

use crate::{
    scenes::{EmulationState, EmulatorState, GameMenuState},
    utils::{layout::*, load_cartridge},
};

pub struct SelectionMenuState {
    pub roms: Vec<PathBuf>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub input: InputManager,
    pub confirming_new_game: bool,
}

impl SelectionMenuState {
    pub fn new(roms_dir: &str) -> Self {
        let mut roms: Vec<PathBuf> = std::fs::read_dir(roms_dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.is_file()
                            && path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.eq_ignore_ascii_case("gb") || ext.eq_ignore_ascii_case("gbc"))
                                .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default();

        roms.sort();

        Self {
            roms,
            selected: 0,
            scroll_offset: 0,
            input: InputManager::with_debounce(0.13),
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
                let cartridge = load_cartridge(&path, save_path)?;
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

        let visible_count = ((selector_bottom() - selector_top()) / ITEM_H) as usize;

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
            let cartridge = load_cartridge(&path, save_path)?;
            *rom_path = Some(path);
            *gb = Some(Dmg::new(cartridge, None));

            return Ok(Some(EmulatorState::Emulation(EmulationState::new())));
        }

        if self.input.is_pressed_right() && !self.confirming_new_game {
            return Ok(Some(EmulatorState::GameMenu(GameMenuState::new())));
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        draw_selector(d, &self.roms, self.selected, self.scroll_offset);

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
                        0 => gbeed_raylib_common::FOREGROUND,
                        3 => gbeed_raylib_common::SECONDARY,
                        _ => gbeed_raylib_common::PRIMARY,
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