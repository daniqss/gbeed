use crate::controller::DebuggerController;
use crate::utils::{BACKGROUND, FOREGROUND, components::draw_button};
use gbeed_raylib_common::input::{InputManager, MouseButtonArea};
use raylib::prelude::*;

#[derive(Default, Debug)]
pub struct WaitingFileScene {
    pub select_btn: MouseButtonArea,
}

impl WaitingFileScene {
    pub fn new() -> Self {
        Self {
            select_btn: MouseButtonArea::default(),
        }
    }

    pub fn update_layout(&mut self, screen_w: i32, screen_h: i32) {
        let (w, h) = (200, 40);
        let (x, y) = ((screen_w - w) / 2, (screen_h - h) / 2 + 50);
        self.select_btn = MouseButtonArea::new(x, y, w, h);
    }

    pub fn update(
        &mut self,
        dt: f32,
        controller: &mut DebuggerController,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if controller.rl.is_file_dropped() {
            let dropped_files = controller.rl.load_dropped_files();
            if let Some(file_path) = dropped_files.iter().next() {
                return Ok(Some(file_path.to_string()));
            }
        }

        #[cfg(target_arch = "wasm32")]
        // SAFETY: We need to call JavaScript to open the file dialog
        unsafe {
            if self.select_btn.is_pressed(&controller.rl, MouseButton::MOUSE_BUTTON_LEFT) {
                let script = std::ffi::CString::new("document.getElementById('rom-input').click()").unwrap();
                crate::web::emscripten_run_script(script.as_ptr());
            }
        }

        Ok(None)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();

        #[cfg(target_arch = "wasm32")]
        let select_msg = "Select a Game Boy ROM";
        let drop_msg = if cfg!(target_arch = "wasm32") {
            "Or drop a ROM to start"
        } else {
            "Drop a Game Boy ROM to start"
        };


        // drop a file message
        let font_size = 20;
        let width = d.measure_text(drop_msg, font_size);
        d.draw_text(
            drop_msg,
            (screen_w - width) / 2,
            (screen_h - font_size) / 2 - 20,
            font_size,
            FOREGROUND,
        );

        // select a file in the file dialog message 
        #[cfg(target_arch = "wasm32")]
        draw_button(d, self.select_btn, select_msg, self.select_btn.is_hovered(d));
    }
}
