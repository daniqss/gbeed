use crate::controller::DebuggerController;
use crate::utils::{BACKGROUND, FOREGROUND};
use gbeed_raylib_common::input::InputManager;
use raylib::prelude::*;

#[derive(Default, Debug)]
pub struct WaitingFileScene {
    pub input: InputManager,
}

pub enum WaitingFileEvent {
    LoadRom(String),
    Exit,
}

impl WaitingFileScene {
    pub fn new() -> Self {
        Self {
            input: InputManager::default(),
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        controller: &mut DebuggerController,
    ) -> Result<Option<WaitingFileEvent>, Box<dyn std::error::Error>> {
        self.input.update(&controller.rl, dt);

        if controller.rl.is_file_dropped() {
            let dropped_files = controller.rl.load_dropped_files();
            if let Some(file_path) = dropped_files.iter().next() {
                return Ok(Some(WaitingFileEvent::LoadRom(file_path.to_string())));
            }
        }

        if self.input.is_pressed_escape() {
            return Ok(Some(WaitingFileEvent::Exit));
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.clear_background(BACKGROUND);
        let msg = "Drag and Drop a Game Boy ROM to start";
        let font_size = 20;
        let width = d.measure_text(msg, font_size);
        d.draw_text(
            msg,
            (d.get_screen_width() - width) / 2,
            (d.get_screen_height() - font_size) / 2,
            font_size,
            FOREGROUND,
        );
    }
}
