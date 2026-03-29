use super::emulation::EmulationState;
use super::EmulatorState;
use crate::utils::layout::*;
use gbeed_raylib_common::{InputKeyTriggers, InputState, ToInputState};
use raylib::prelude::*;
use std::path::PathBuf;

pub struct SelectionMenuState {
    pub roms: Vec<PathBuf>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub repeat_timer: f32,
    pub triggers: InputKeyTriggers,
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
            triggers: InputKeyTriggers::default(),
            last_input: InputState::default(),
        }
    }

    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        dt: f32,
        rom_path: &mut Option<PathBuf>,
    ) -> Option<EmulatorState> {
        let input = self.triggers.to_input(rl);

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

        let a_pressed = input.a && !self.last_input.a;
        self.last_input = input;

        if a_pressed && !self.roms.is_empty() {
            *rom_path = Some(self.roms[self.selected].clone());
            Some(EmulatorState::Emulation(EmulationState {
                key_triggers: InputKeyTriggers::default(),
            }))
        } else {
            None
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        draw_header(d, self.roms.len());
        draw_selector(d, &self.roms, self.selected, self.scroll_offset);
    }
}
