mod controller;
mod scenes;
mod utils;

use gbeed_core::prelude::*;
use gbeed_raylib_common::{Palette, Texture};
use raylib::prelude::*;
use std::path::PathBuf;
use std::process::exit;

use crate::controller::ConsoleController;
use crate::scenes::{EmulatorState, SelectionMenuState};
use crate::utils::layout::{draw_footer, draw_header, SCREEN_HEIGHT, SCREEN_WIDTH};

struct EmulatorApp {
    state: EmulatorState,
    gb: Option<Dmg>,
    rom_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
    controller: ConsoleController,
}

impl EmulatorApp {
    pub fn new(mut rl: RaylibHandle, thread: RaylibThread) -> Self {
        let screen = Texture::new(
            &mut rl,
            &thread,
            DMG_SCREEN_WIDTH as i32,
            DMG_SCREEN_HEIGHT as i32,
        );

        Self {
            state: EmulatorState::SelectionMenu(SelectionMenuState::new()),
            gb: None,
            rom_path: None,
            save_path: None,

            controller: ConsoleController {
                rl,
                thread,
                screen,
                palette: Palette::default(),
            },
        }
    }

    #[inline(always)]
    pub fn should_close(&self) -> bool {
        self.controller.rl.window_should_close() || matches!(self.state, EmulatorState::Exit)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dt = self.controller.rl.get_frame_time();

        let next_state = match &mut self.state {
            EmulatorState::SelectionMenu(state) => state.update(
                &self.controller.rl,
                dt,
                &mut self.rom_path,
                &mut self.gb,
                &mut self.save_path,
            )?,
            EmulatorState::Emulation(state) => state.update(
                dt,
                &mut self.gb,
                &mut self.rom_path,
                &mut self.save_path,
                &mut self.controller,
            )?,
            EmulatorState::GameMenu(state) => state.update(&self.controller.rl, dt, &self.gb),
            EmulatorState::SettingsMenu(state) => state.update(dt, &mut self.controller),

            // emulator should have already been closed at this point
            EmulatorState::Exit => unreachable!(),
        };

        if let Some(state) = next_state {
            self.state = state;
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        let palette = self.controller.palette;
        let ConsoleController {
            rl, thread, screen, ..
        } = &mut self.controller;

        rl.draw(thread, |mut d| {
            d.clear_background(palette.background());

            match &self.state {
                EmulatorState::SelectionMenu(state) => state.draw(&mut d, palette),
                EmulatorState::Emulation(state) => state.draw(&mut d, screen),
                EmulatorState::GameMenu(state) => {
                    state.draw(&mut d, screen, &self.gb, &self.rom_path, palette)
                }
                EmulatorState::SettingsMenu(state) => state.draw(&mut d, palette),

                EmulatorState::Exit => return,
            }

            draw_header(&mut d, &self.state, palette);
            draw_footer(&mut d, &self.state, palette);
            d.draw_fps(215, 220);
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("gbeed")
        .build();
    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut app = EmulatorApp::new(rl, thread);

    while !app.should_close() {
        app.update()?;
        app.draw();
    }

    exit(0)
}
