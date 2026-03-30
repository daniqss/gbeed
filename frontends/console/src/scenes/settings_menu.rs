use crate::scenes::{EmulatorState, GameMenuState};
use gbeed_raylib_common::{InputKeyTriggers, InputState, ToInputState};
use raylib::prelude::*;

#[derive(Default)]
pub struct SettingsMenuState {
    pub key_triggers: InputKeyTriggers,
    pub last_input: InputState,
}

impl SettingsMenuState {
    pub fn update(&mut self, rl: &RaylibHandle) -> Option<EmulatorState> {
        let input = self.key_triggers.to_input(rl);
        let left_pressed = input.left && !self.last_input.left;
        self.last_input = input;

        if left_pressed {
            return Some(EmulatorState::GameMenu(GameMenuState::default()));
        }

        None
    }

    pub fn draw(&self, _d: &mut RaylibDrawHandle) {
        // TODO: Implement settings menu
    }
}
