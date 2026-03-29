use raylib::prelude::*;

pub struct SettingsMenuState;

impl SettingsMenuState {
    pub fn update(&mut self, _rl: &RaylibHandle) -> Option<super::EmulatorState> {
        None
    }

    pub fn draw(&self, _d: &mut RaylibDrawHandle) {
        // draw settings menu
    }
}
