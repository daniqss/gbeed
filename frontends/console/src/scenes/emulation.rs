use crate::controller::ConsoleController;
use crate::scenes::EmulatorState;
use crate::scenes::GameMenuState;
use crate::utils::layout::*;
use crate::utils::roms::save_cartridge;
use gbeed_core::prelude::*;
use gbeed_raylib_common::input::InputManager;
use raylib::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EmulationState {
    pub input: InputManager,
}

impl EmulationState {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        gb: &mut Option<Dmg>,
        _rom_path: &mut Option<PathBuf>,
        save_path: &mut Option<PathBuf>,
        controller: &mut ConsoleController,
    ) -> Result<Option<EmulatorState>, Box<dyn std::error::Error>> {
        let Some(gb) = gb else {
            return Ok(None);
        };

        self.input.update(&controller.rl, dt);

        if self.input.is_pressed_escape() {
            save_cartridge(gb, save_path)?;
            return Ok(Some(EmulatorState::GameMenu(GameMenuState::new())));
        }

        self.input.state().apply(&mut gb.joypad);

        gb.run(controller)?;
        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, screen: &gbeed_raylib_common::Texture) {
        d.draw_texture_pro(
            &screen.texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}
