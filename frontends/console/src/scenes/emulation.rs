use crate::controller::ConsoleController;
use crate::utils::layout::*;
use crate::utils::save_path_from_rom;
use gbeed_core::prelude::*;
use gbeed_raylib_common::{InputKeyTriggers, ToInputState};
use raylib::prelude::*;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct EmulationState {
    pub key_triggers: InputKeyTriggers,
}

impl EmulationState {
    pub fn update(
        &mut self,
        gb: &mut Option<Dmg>,
        rom_path: &mut Option<PathBuf>,
        save_path: &mut Option<PathBuf>,
        controller: &mut ConsoleController,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(gb) = gb else {
            if let Some(game_path) = rom_path {
                *gb = Some(Dmg::new(self.load_rom(game_path, save_path)?, None));
            }
            return Ok(());
        };

        let input = self.key_triggers.to_input(&controller.rl);
        input.apply(&mut gb.joypad);

        gb.run(controller)
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

    fn load_rom(
        &mut self,
        game_path: &PathBuf,
        save_path: &mut Option<PathBuf>,
    ) -> Result<Cartridge, Box<dyn std::error::Error>> {
        let s_path = save_path_from_rom(game_path.to_str().unwrap_or_default());
        *save_path = Some(s_path.clone());

        let game_data = fs::read(game_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to read game ROM at {game_path:?}: {e}"),
            )
        })?;

        let save = match fs::read(&s_path) {
            Ok(data) => Some(data),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => {
                return Err(Box::new(io::Error::other(format!(
                    "Failed to read save file at {:?}: {e}",
                    s_path
                ))))
            }
        };

        Ok(Cartridge::new(&game_data, save).map_err(|e| {
            Box::new(std::io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to create cartridge from ROM at {game_path:?}: {e}"),
            ))
        })?)
    }
}
