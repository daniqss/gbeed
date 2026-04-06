use gbeed_core::prelude::*;
use gbeed_raylib_common::input::{InputManager, InputMouseTriggers, MouseButtonArea};

mod controller;
#[cfg(target_arch = "wasm32")]
mod web;

use raylib::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use controller::{renderer, RaylibController};
#[cfg(target_arch = "wasm32")]
use web::{
    emscripten_set_main_loop_arg, load_rom_from_js, local_storage, save_game_wasm, wasm_main_loop, APP_PTR,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut game_path: Option<String> = None;
    let mut boot_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-g" | "--game" => {
                if i + 1 < args.len() {
                    game_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-b" | "--boot" | "--boot_rom" => {
                if i + 1 < args.len() {
                    boot_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    #[cfg(target_arch = "wasm32")]
    let (window_width, window_height, is_mobile) = unsafe {
        let script_w = std::ffi::CString::new(
            "Math.max(document.documentElement.clientWidth || 0, window.innerWidth || 0)",
        )
        .unwrap();
        let script_h = std::ffi::CString::new(
            "Math.max(document.documentElement.clientHeight || 0, window.innerHeight || 0)",
        )
        .unwrap();
        let w = web::emscripten_run_script_int(script_w.as_ptr());
        let h = web::emscripten_run_script_int(script_h.as_ptr());

        let script_m = std::ffi::CString::new("(/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) || (window.innerWidth / window.innerHeight) < 1.0) ? 1 : 0").unwrap();
        let m = web::emscripten_run_script_int(script_m.as_ptr()) != 0;
        (w, h, m)
    };

    #[cfg(not(target_arch = "wasm32"))]
    let (window_width, window_height, is_mobile) = (1920, 1080, false);

    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("gbeed")
        .resizable()
        .build();
    rl.set_target_fps(60);

    let mut app = EmulatorApp::new(rl, thread, boot_path, is_mobile, window_width, window_height);

    if let Some(path) = game_path {
        if let Err(e) = app.load_rom(&path) {
            eprintln!("Failed to load ROM from args: {e}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        while !app.controller.renderer.rl.window_should_close() && !app.input.state().escape {
            app.update()?;
        }

        if let Err(e) = app.save_game() {
            eprintln!("Failed to save game on exit: {e}");
        }
    }

    // Force the linker to keep save_game_wasm
    #[cfg(target_arch = "wasm32")]
    let _ = save_game_wasm as *const ();
    #[cfg(target_arch = "wasm32")]
    let _ = load_rom_from_js as *const ();

    // set up the wasm main loop
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let app_ptr = Box::into_raw(Box::new(app));
        APP_PTR = app_ptr;
        emscripten_set_main_loop_arg(wasm_main_loop, app_ptr, 0, 1);
    }

    Ok(())
}

#[repr(C)]
pub struct EmulatorApp {
    gb: Option<Dmg>,
    controller: RaylibController,
    save_path: Option<PathBuf>,
    boot_rom: Option<Vec<u8>>,
    input: InputManager,
}

impl EmulatorApp {
    fn new(
        rl: RaylibHandle,
        thread: RaylibThread,
        boot_path: Option<String>,
        is_mobile: bool,
        window_width: i32,
        window_height: i32,
    ) -> Self {
        let boot_rom = boot_path.and_then(|path| fs::read(path).ok());
        let layout = renderer::Layout::new(window_width, window_height, is_mobile);
        let controller = RaylibController::new(rl, thread, layout);

        let mouse_triggers = InputMouseTriggers {
            up: MouseButtonArea::new(layout.dpad_x - 17, layout.dpad_y - layout.dpad_arm - 17, 34, 34),
            down: MouseButtonArea::new(layout.dpad_x - 17, layout.dpad_y + layout.dpad_arm - 17, 34, 34),
            left: MouseButtonArea::new(layout.dpad_x - layout.dpad_arm - 17, layout.dpad_y - 17, 34, 34),
            right: MouseButtonArea::new(layout.dpad_x + layout.dpad_arm - 17, layout.dpad_y - 17, 34, 34),
            select: MouseButtonArea::new(
                layout.start_select_x,
                layout.start_select_y,
                layout.start_select_width,
                20,
            ),
            start: MouseButtonArea::new(
                layout.start_select_x + layout.start_select_width + 18,
                layout.start_select_y,
                layout.start_select_width,
                20,
            ),
            a: MouseButtonArea::new(layout.action_buttons_x - 18 + 24, layout.action_buttons_y, 36, 36),
            b: MouseButtonArea::new(
                layout.action_buttons_x - 18 - 24,
                layout.action_buttons_y + 44,
                36,
                36,
            ),
            speed_up: Some(MouseButtonArea::new(
                layout.screen_center_x - 118 / 2,
                layout.controls_y - 20,
                118,
                26,
            )),
            ..Default::default()
        };

        let input = InputManager::new(0.1, None, Some(mouse_triggers), None);

        Self {
            gb: None,
            controller,
            save_path: None,
            boot_rom,
            input,
        }
    }

    fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let game_data = fs::read(path)?;

        let save_path = if cfg!(target_arch = "wasm32") {
            // in web we need to clean the /.glfw_dropped_files/ directory
            PathBuf::from(format!(
                "saves/{}",
                save_path_from_rom(path.split('/').next_back().unwrap_or("")).to_string_lossy()
            ))
        } else {
            save_path_from_rom(path)
        };

        #[cfg(target_arch = "wasm32")]
        let save = local_storage::load_save(&save_path);

        #[cfg(not(target_arch = "wasm32"))]
        let save = match fs::read(&save_path) {
            Ok(data) => {
                println!("Loaded save file from {}", save_path.display());
                Some(data)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("No save file found at {}, starting fresh", save_path.display());
                None
            }
            Err(e) => return Err(Box::new(e)),
        };

        let game = Cartridge::new(&game_data, save).map_err(|e| format!("{e}"))?;
        self.controller
            .renderer
            .set_game_info(game.header.title.clone(), game.header.destination);

        self.gb = Some(Dmg::new(game, self.boot_rom.clone()));
        self.save_path = Some(save_path);

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Handle Drag and Drop
        if self.controller.renderer.rl.is_file_dropped() {
            let dropped_files = self.controller.renderer.rl.load_dropped_files();
            if let Some(file_path) = dropped_files.iter().next() {
                if let Err(e) = self.load_rom(file_path) {
                    eprintln!("Failed to load dropped ROM: {e}");
                }
            }
        }

        let dt = self.controller.renderer.rl.get_frame_time();
        self.input.update(&self.controller.renderer.rl, dt);
        self.controller.renderer.buttons = self.input.state();

        if self.input.is_pressed_speed_up() {
            self.controller.renderer.cycle_fps();
        }

        if let Some(ref mut gb) = self.gb {
            self.input.state().apply(&mut gb.joypad);

            gb.run(&mut self.controller)?;

            renderer::update_tiles(
                &mut self.controller.renderer.tile_textures[0],
                gb.ppu.tile_block0(),
            );
            renderer::update_tiles(
                &mut self.controller.renderer.tile_textures[1],
                gb.ppu.tile_block1(),
            );
            renderer::update_tiles(
                &mut self.controller.renderer.tile_textures[2],
                gb.ppu.tile_block2(),
            );

            renderer::update_bg_map(
                &mut self.controller.renderer.bg_map_texture,
                gb.ppu.bg_map0(),
                gb.ppu.tile_data(),
                gb.ppu.bg_tile_map_address(),
                gb.ppu.get_bg_palette(),
            );

            self.controller
                .renderer
                .update_scroll(gb.read(0xFF43) as i32, gb.read(0xFF42) as i32);
        } else {
            // Draw a "Drop ROM" message
            let mut d = self
                .controller
                .renderer
                .rl
                .begin_drawing(&self.controller.renderer.thread);
            d.clear_background(renderer::BACKGROUND);
            let msg = "Drag and Drop a Game Boy ROM to start";
            let font_size = 20;
            let width = d.measure_text(msg, font_size);
            d.draw_text(
                msg,
                (d.get_screen_width() - width) / 2,
                (d.get_screen_height() - font_size) / 2,
                font_size,
                renderer::FOREGROUND,
            );
        }
        Ok(())
    }

    fn save_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let gb = self.gb.as_ref().ok_or("No game loaded")?;
        let save_data = gb.cartridge.save_game().ok_or("Game does not support saving")?;
        let save_path = self.save_path.as_ref().ok_or("Save path not set")?;

        #[cfg(target_arch = "wasm32")]
        local_storage::store_save(save_path, save_data);

        #[cfg(not(target_arch = "wasm32"))]
        {
            fs::write(save_path, save_data)?;
            println!("Game saved successfully to {}", save_path.display());
        }

        Ok(())
    }
}

fn save_path_from_rom(rom_path: &str) -> PathBuf {
    let path = Path::new(rom_path);
    match path.extension().and_then(|e| e.to_str()) {
        Some("gb" | "gbc") => path.with_extension("sav"),
        _ => path.with_added_extension("sav"),
    }
}

fn print_help() {
    println!("Usage: gbeed [OPTIONS]");
    println!("Options:");
    println!("  -g, --game <PATH>      Path to the game ROM file");
    println!("  -b, --boot <PATH>      Path to the boot ROM file (optional)");
    println!("  -h, --help             Print this help message");
}
