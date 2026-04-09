mod controller;
mod scenes;
mod utils;

use gbeed_core::prelude::*;
use gbeed_raylib_common::{
    Texture, color,
    settings::{SpeedUpMode, SpeedUpMultiplier, TargetedFps},
};
use raylib::prelude::*;
use std::path::PathBuf;

use crate::controller::ConsoleController;
use crate::scenes::{EmulatorState, SelectionMenuState};
use crate::utils::layout::{SCREEN_HEIGHT, SCREEN_WIDTH, draw_footer, draw_header};

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

        let palette = color::Palette::default();

        Self {
            state: EmulatorState::SelectionMenu(SelectionMenuState::new()),
            gb: None,
            rom_path: None,
            save_path: None,

            controller: ConsoleController {
                screen,
                palette,
                palette_color: palette.get_palette_color(),
                speed_up_mode: SpeedUpMode::default(),
                speed_up_multiplier: SpeedUpMultiplier::default(),
                targeted_fps: TargetedFps::default(),

                rl,
                thread,
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
            EmulatorState::SettingsMenu(state) => state.update(dt, self.gb.as_ref(), &mut self.controller),

            // emulator should have already been closed at this point
            EmulatorState::Exit => unreachable!(),
        };

        if let Some(state) = next_state {
            self.state = state;
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        let ConsoleController {
            rl,
            thread,
            screen,
            palette,
            palette_color,
            speed_up_mode,
            speed_up_multiplier,
            targeted_fps,
            ..
        } = &mut self.controller;

        rl.draw(thread, |mut d| {
            d.clear_background(color::background(palette_color));

            match &self.state {
                EmulatorState::SelectionMenu(state) => state.draw(&mut d, palette_color),
                EmulatorState::Emulation(state) => state.draw(&mut d, screen),
                EmulatorState::GameMenu(state) => {
                    state.draw(&mut d, screen, &self.gb, &self.rom_path, palette_color)
                }
                EmulatorState::SettingsMenu(state) => state.draw(
                    &mut d,
                    palette,
                    palette_color,
                    speed_up_mode,
                    speed_up_multiplier,
                    targeted_fps,
                ),

                EmulatorState::Exit => return,
            }

            draw_header(&mut d, &self.state, palette_color);
            draw_footer(&mut d, &self.state, palette_color);
            d.draw_fps(215, 220);
            // if let Some(gb) = &mut self.gb {
            //     d.draw_text(
            //         &format!("{}", gb.ppu.sprites_this_frame),
            //         205,
            //         200,
            //         16,
            //         Color::GREENYELLOW,
            //     );

            //     gb.ppu.sprites_this_frame = 0;
            // }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("gbeed")
        .build();
    rl.set_target_fps(30);
    rl.set_exit_key(None);

    let mut app = EmulatorApp::new(rl, thread);

    while !app.should_close() {
        app.update()?;
        app.draw();
    }

    Ok(())
}
