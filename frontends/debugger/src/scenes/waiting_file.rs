use crate::controller::DebuggerController;
use crate::utils::{BACKGROUND, FOREGROUND, components::draw_button};
use crate::web;
use gbeed_core::prelude::*;
use gbeed_raylib_common::input::{MouseButtonArea};
use raylib::prelude::*;

#[derive(Default, Debug)]
pub struct WaitingFileScene {
    pub select_btn: MouseButtonArea,
    pub was_button_pressed: bool,
}

impl WaitingFileScene {
    pub fn new() -> Self {
        Self {
            select_btn: MouseButtonArea::default(),
            was_button_pressed: false,
        }
    }

    pub fn update_layout(&mut self, screen_w: i32, screen_h: i32) {
        let (w, h) = (200, 40);
        let x = (screen_w - w) / 2;
        let y = (screen_h - h) / 2 - 40;

        self.select_btn = MouseButtonArea::new(x, y, w, h);
    }

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


        #[cfg(target_arch = "wasm32")]
        {
            let is_down = self.select_btn.is_pressed(&controller.rl, MouseButton::MOUSE_BUTTON_LEFT);
            if is_down && !self.was_button_pressed {
                self.was_button_pressed = true;
                unsafe {
                    web::open_file_dialog();
                }
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
            (screen_h - font_size) / 2 + 40,
            font_size,
            FOREGROUND,
        );

        #[cfg(target_arch = "wasm32")]
        {
            draw_button(d, &self.select_btn, "Select ROM");
        }
    }
}
