use gbeed_core::prelude::*;
use raylib::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

mod controller;
mod scenes;
mod utils;

#[cfg(target_arch = "wasm32")]
mod web;

use controller::DebuggerController;
use scenes::{EmulationScene, EmulatorState, WaitingFileScene};
use utils::{BACKGROUND, Layout};

#[cfg(target_arch = "wasm32")]
use web::{
    APP_PTR, emscripten_set_main_loop_arg, load_rom_from_js, local_storage, save_game_wasm, wasm_main_loop,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut game_path = None;
    let mut boot_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-g" | "--game" if i + 1 < args.len() => {
                game_path = Some(args[i + 1].clone());
                i += 1;
            }
            "-b" | "--boot" | "--boot_rom" if i + 1 < args.len() => {
                boot_path = Some(args[i + 1].clone());
                i += 1;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    let (width, height, is_mobile) = get_platform_info();

    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("gbeed")
        .resizable()
        .build();

    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut app = EmulatorApp::new(&mut rl, &thread, boot_path, is_mobile);

    // load ROM if its provided via command line args
    if let Some(path) = game_path {
        match app.load_rom(&path) {
            Ok(state) => app.state = state,
            Err(e) => eprintln!("Failed to load ROM from args: {e}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        while !app.should_close() {
            app.update()?;
            app.draw();
        }
        let _ = app.save_game();
    }

    #[cfg(target_arch = "wasm32")]
    unsafe {
        // force the linker to keep these functions
        let _ = save_game_wasm as *const ();
        let _ = load_rom_from_js as *const ();

        let app_ptr = Box::into_raw(Box::new(app));
        APP_PTR = app_ptr;
        emscripten_set_main_loop_arg(wasm_main_loop, app_ptr, 0, 1);
    }

    Ok(())
}

fn get_platform_info() -> (i32, i32, bool) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
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
        let is_mobile = web::emscripten_run_script_int(script_m.as_ptr()) != 0;
        (w, h, is_mobile)
    }

    #[cfg(not(target_arch = "wasm32"))]
    (1920, 1080, false)
}

#[repr(C)]
pub struct EmulatorApp<'a> {
    gb: Option<Dmg>,
    controller: DebuggerController<'a>,
    save_path: Option<PathBuf>,
    boot_rom: Option<Vec<u8>>,
    state: EmulatorState,
    layout: Layout,
}

impl<'a> EmulatorApp<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        boot_path: Option<String>,
        is_mobile: bool,
    ) -> Self {
        let boot_rom = boot_path.and_then(|path| fs::read(path).ok());
        let (sw, sh) = (rl.get_screen_width(), rl.get_screen_height());
        let layout = Layout::new(sw, sh, is_mobile);
        let controller = DebuggerController::new(rl, thread);
        let mut scene = WaitingFileScene::new();
        scene.update_layout(sw, sh);
        let state = EmulatorState::WaitingFile(scene);

        Self {
            gb: None,
            controller,
            save_path: None,
            boot_rom,
            state,
            layout,
        }
    }

    pub fn should_close(&self) -> bool { self.controller.rl.window_should_close() }

    pub fn init_audio(&mut self) { self.controller.init_audio(); }

    pub fn load_rom(&mut self, path: &str) -> Result<EmulatorState, Box<dyn std::error::Error>> {
        let game_data = fs::read(path)?;

        let save_path = if cfg!(target_arch = "wasm32") {
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
            _ => None,
        };

        let game = Cartridge::new(&game_data, save).map_err(|e| format!("{e}"))?;
        let title = game.header.title.clone();
        let region = format!("{:?}", game.header.destination);

        self.gb = Some(Dmg::new(game, self.boot_rom.clone()));
        self.save_path = Some(save_path);

        Ok(EmulatorState::Emulation(EmulationScene::new(
            self.layout,
            title,
            region,
        )))
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dt = self.controller.rl.get_frame_time();

        // handle window resizing
        if self.controller.rl.is_window_resized() {
            let (sw, sh) = (
                self.controller.rl.get_screen_width(),
                self.controller.rl.get_screen_height(),
            );
            self.layout = Layout::new(sw, sh, self.layout.is_mobile);
            match &mut self.state {
                EmulatorState::Emulation(scene) => scene.update_layout(self.layout),
                EmulatorState::WaitingFile(scene) => scene.update_layout(sw, sh),
            }
        }

        let next_state = match &mut self.state {
            EmulatorState::WaitingFile(scene) => match scene.update(&mut self.controller) {
                Ok(Some(path)) => self.load_rom(&path).ok(),
                _ => None,
            },
            EmulatorState::Emulation(scene) => {
                scene.scroll_x = self.controller.scroll_x;
                scene.scroll_y = self.controller.scroll_y;
                scene.update(dt, self.gb.as_mut(), &mut self.controller)?
            }
        };

        if let Some(state) = next_state {
            self.state = state;
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        let state = &self.state;
        let controller = &mut self.controller;

        controller.rl.draw(controller.thread, |mut d| {
            d.clear_background(BACKGROUND);

            match state {
                EmulatorState::WaitingFile(scene) => scene.draw(&mut d),
                EmulatorState::Emulation(scene) => scene.draw(
                    &mut d,
                    &controller.screen_texture,
                    &controller.tile_textures,
                    &controller.bg_map_texture,
                ),
            }
        });
    }

    pub fn save_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (gb, save_path) = match (self.gb.as_ref(), self.save_path.as_ref()) {
            (Some(gb), Some(path)) => (gb, path),
            _ => return Ok(()),
        };

        if let Some(save_data) = gb.cartridge.save_game() {
            #[cfg(target_arch = "wasm32")]
            local_storage::store_save(save_path, save_data);

            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(parent) = save_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(save_path, save_data)?;
                println!("Game saved successfully to {}", save_path.display());
            }
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
