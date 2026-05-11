use crate::controller::ConsoleController;
use crate::scenes::EmulatorState;
use crate::scenes::GameMenuState;
use crate::utils::layout::*;
use crate::utils::roms::save_cartridge;
use gbeed_core::prelude::*;
use gbeed_raylib_common::{input::InputManager, settings::SpeedUpMode};
use raylib::prelude::*;
use std::path::PathBuf;

const GB_FRAME_TIME: f32 = 1.0 / 59.73;
const MAX_STEPS: usize = 5;
const MAX_ACCUMULATOR: f32 = GB_FRAME_TIME * MAX_STEPS as f32;

#[derive(Debug)]
pub struct EmulationState {
    pub input: InputManager,
    pub accumulator: f32,
}

impl EmulationState {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
            accumulator: 0.0,
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

        self.input.update(controller.rl, dt);

        if self.input.is_pressed_escape() {
            save_cartridge(gb, save_path)?;
            return Ok(Some(EmulatorState::GameMenu(GameMenuState::new())));
        }

        self.input.state().apply(&mut gb.joypad);

        let selected_speed_up = controller.speed_up_multiplier.get_multiplier();

        let speed = match &mut controller.speed_up_mode {
            SpeedUpMode::Hold if self.input.is_held_speed_up() => selected_speed_up,
            SpeedUpMode::Hold => 1.0,

            SpeedUpMode::Toggle(active) => {
                if self.input.is_pressed_speed_up() {
                    *active = !*active;
                }

                if *active { selected_speed_up } else { 1.0 }
            }
        };

        // avoid spiral of death by capping the accumulator if the hardware can't keep up
        self.accumulator += dt * speed;
        self.accumulator = self.accumulator.min(MAX_ACCUMULATOR);

        let mut steps = 0;

        while self.accumulator >= GB_FRAME_TIME && steps < MAX_STEPS {
            gb.run(controller)?;
            self.accumulator -= GB_FRAME_TIME;
            steps += 1;
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, screen: &gbeed_raylib_common::Texture) {
        d.draw_texture_pro(
            screen,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
        // d.draw_texture(&screen.texture, 0, 0, Color::WHITE);
    }
}
