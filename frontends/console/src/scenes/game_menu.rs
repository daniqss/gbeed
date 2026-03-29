use raylib::prelude::*;

pub struct GameMenuState;

impl GameMenuState {
    pub fn update(&mut self, _rl: &RaylibHandle) -> Option<super::EmulatorState> {
        None
    }

    pub fn draw(&self, _d: &mut RaylibDrawHandle) {
        // draw game menu
    }
}
