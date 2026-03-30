use gbeed_core::prelude::Dmg;
use gbeed_raylib_common::{InputKeyTriggers, InputState, ToInputState};
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
    pub repeat_timer: f32,
    pub debounce_timer: f32,
    pub triggers: InputKeyTriggers,
    pub confirming_new_game: bool,
    last_input: InputState,
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
            repeat_timer: 0.0,
            debounce_timer: 0.13,
            triggers: InputKeyTriggers::default(),
            confirming_new_game: false,
            last_input: InputState::default(),
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
        if self.debounce_timer > 0.0 {
            self.debounce_timer -= dt;
            return Ok(None);
        }

        let input = self.triggers.to_input(rl);

        let a_pressed = input.a && !self.last_input.a;
        let right_pressed = input.right && !self.last_input.right;

        if self.confirming_new_game {
            let b_pressed = input.b && !self.last_input.b;
            let start_pressed = input.start && !self.last_input.start;

            self.last_input = input;

            if a_pressed || start_pressed {
                self.confirming_new_game = false;
                let path = self.roms[self.selected].clone();
                let cartridge = load_cartridge(&path, save_path)?;
                *rom_path = Some(path);
                *gb = Some(Dmg::new(cartridge, None));

                return Ok(Some(EmulatorState::Emulation(EmulationState {
                    key_triggers: InputKeyTriggers::default(),
                })));
            } else if b_pressed {
                self.confirming_new_game = false;
            }
            return Ok(None);
        }

        let up_held = input.up;
        let down_held = input.down;
        let up_pressed = input.up && !self.last_input.up;
        let down_pressed = input.down && !self.last_input.down;

        let mut move_up = up_pressed;
        let mut move_down = down_pressed;

        const REPEAT_DELAY: f32 = 0.3;
        const REPEAT_RATE: f32 = 0.08;

        if up_held || down_held {
            self.repeat_timer += dt;
            if up_pressed || down_pressed {
                self.repeat_timer = 0.0;
            }
            if self.repeat_timer >= REPEAT_DELAY {
                let ticks = ((self.repeat_timer - REPEAT_DELAY) / REPEAT_RATE) as usize;
                let prev = ((self.repeat_timer - REPEAT_DELAY - dt.max(0.0)) / REPEAT_RATE) as usize;
                if ticks > prev {
                    if up_held {
                        move_up = true;
                    }
                    if down_held {
                        move_down = true;
                    }
                }
            }
        } else {
            self.repeat_timer = 0.0;
        }

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

        self.last_input = input;

        if a_pressed && !self.roms.is_empty() {
            if gb.is_some() {
                self.confirming_new_game = true;
                return Ok(None);
            }

            let path = self.roms[self.selected].clone();
            let cartridge = load_cartridge(&path, save_path)?;
            *rom_path = Some(path);
            *gb = Some(Dmg::new(cartridge, None));

            return Ok(Some(EmulatorState::Emulation(EmulationState {
                key_triggers: InputKeyTriggers::default(),
            })));
        }

        if right_pressed && !self.confirming_new_game {
            return Ok(Some(EmulatorState::GameMenu(GameMenuState::default())));
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
