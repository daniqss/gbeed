use crate::scenes::{EmulatorState, GameMenuState};
use gbeed_raylib_common::InputManager;
use raylib::prelude::*;

pub struct SettingsMenuState {
    pub input: InputManager,
}

impl SettingsMenuState {
    pub fn new() -> Self {
        Self {
            input: InputManager::with_debounce(0.08),
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32) -> Option<EmulatorState> {
        self.input.update(rl, dt);

        if self.input.is_pressed_left() {
            return Some(EmulatorState::GameMenu(GameMenuState::new()));
        }

        None
    }

    pub fn draw(&self, _d: &mut RaylibDrawHandle) {
        // TODO: Implement settings menu
    }
}
