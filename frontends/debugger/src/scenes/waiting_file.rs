use crate::controller::DebuggerController;
use crate::utils::{BACKGROUND, FOREGROUND};
use raylib::prelude::*;

#[derive(Default, Debug)]
pub struct WaitingFileScene;

impl WaitingFileScene {
    pub fn new() -> Self { Self }

    pub fn update_layout(&mut self, _screen_w: i32, _screen_h: i32) {}

    pub fn update(
        &mut self,
        controller: &mut DebuggerController,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if controller.rl.is_file_dropped() {
            let dropped_files = controller.rl.load_dropped_files();
            if let Some(file_path) = dropped_files.iter().next() {
                return Ok(Some(file_path.to_string()));
            }
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.clear_background(BACKGROUND);

        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();

        #[cfg(target_arch = "wasm32")]
        let msg = "Select a Game Boy ROM to start";
        #[cfg(not(target_arch = "wasm32"))]
        let msg = "Drag and Drop a Game Boy ROM to start";

        let font_size = 20;
        let width = d.measure_text(msg, font_size);
        d.draw_text(
            msg,
            (screen_w - width) / 2,
            (screen_h - font_size) / 2,
            font_size,
            FOREGROUND,
        );
    }
}
