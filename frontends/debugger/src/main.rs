use gbeed_core::prelude::*;

mod controller;
mod input;

use raylib::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use controller::{renderer, RaylibController};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    let (mut rl, thread) = raylib::init().size(1920, 1080).title("gbeed").resizable().build();
    rl.set_target_fps(60);

    let mut app = EmulatorApp::new(rl, thread, boot_path);

    // Force the linker to keep save_game_wasm
    #[cfg(target_arch = "wasm32")]
    let _ = save_game_wasm as *const ();

    if let Some(path) = game_path {
        if let Err(e) = app.load_rom(&path) {
            eprintln!("Failed to load ROM from args: {e}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        while !app.controller.renderer.rl.window_should_close()
            && !app.controller.renderer.rl.is_key_down(KeyboardKey::KEY_ESCAPE)
        {
            app.update()?;
        }

        if let Err(e) = app.save_game() {
            eprintln!("Failed to save game on exit: {e}");
        }
    }

    #[cfg(target_arch = "wasm32")]
    unsafe {
        let app_ptr = Box::into_raw(Box::new(app));
        APP_PTR = app_ptr;
        emscripten_set_main_loop_arg(wasm_main_loop, app_ptr, 0, 1);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
static mut APP_PTR: *mut EmulatorApp = std::ptr::null_mut();

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn save_game_wasm() {
    unsafe {
        if !APP_PTR.is_null() {
            let _ = (*APP_PTR).save_game();
        }
    }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn emscripten_set_main_loop_arg(
        func: unsafe extern "C" fn(*mut EmulatorApp),
        arg: *mut EmulatorApp,
        fps: std::os::raw::c_int,
        simulate_infinite_loop: std::os::raw::c_int,
    );
    fn emscripten_run_script(script: *const std::os::raw::c_char);
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" fn wasm_main_loop(app: *mut EmulatorApp) {
    let app = &mut *app;
    if let Err(e) = app.update() {
        eprintln!("Error during update: {e}");
    }
}

#[repr(C)]
struct EmulatorApp {
    gb: Option<Dmg>,
    controller: RaylibController,
    save_path: Option<PathBuf>,
    boot_rom: Option<Vec<u8>>,
}

impl EmulatorApp {
    fn new(rl: RaylibHandle, thread: RaylibThread, boot_path: Option<String>) -> Self {
        let boot_rom = boot_path.and_then(|path| fs::read(path).ok());
        let controller = RaylibController::new(rl, thread);
        Self {
            gb: None,
            controller,
            save_path: None,
            boot_rom,
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
        let save = load_from_local_storage(&save_path);

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

    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Handle Drag and Drop
        if self.controller.renderer.rl.is_file_dropped() {
            let dropped_files = self.controller.renderer.rl.load_dropped_files();
            if let Some(file_path) = dropped_files.iter().next() {
                if let Err(e) = self.load_rom(file_path) {
                    eprintln!("Failed to load dropped ROM: {e}");
                }
            }
        }

        if let Some(ref mut gb) = self.gb {
            input::update(&mut self.controller.renderer, &mut gb.joypad);

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
        save_to_local_storage(save_path, save_data);

        #[cfg(not(target_arch = "wasm32"))]
        {
            fs::write(save_path, save_data)?;
            println!("Game saved successfully to {}", save_path.display());
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn save_to_local_storage(path: &Path, data: &[u8]) {
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");

    // use of MEMFS to bridge the save data to JavaScript
    let tmp_path = "/tmp_save.sav";
    if let Err(e) = fs::write(tmp_path, data) {
        eprintln!("Failed to write bridge file: {:?}", e);
        return;
    }

    let script = format!(
        "try {{ \
        const bytes = FS.readFile('{}'); \
        const binary = Array.from(bytes, b => String.fromCharCode(b)).join(''); \
        const encoded = btoa(binary); \
        localStorage.setItem('{}', encoded); \
        console.log('Game saved to localStorage: {}'); \
    }} catch (e) {{ \
        console.error('Error saving to localStorage:', e); \
    }}",
        tmp_path, filename, filename
    );

    match std::ffi::CString::new(script) {
        Ok(script) => unsafe {
            emscripten_run_script(script.as_ptr());
        },
        _ => eprintln!("Failed to create script string for saving to localStorage"),
    };
    let _ = fs::remove_file(tmp_path);
}

#[cfg(target_arch = "wasm32")]
fn load_from_local_storage(path: &Path) -> Option<Vec<u8>> {
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("unknown");
    let tmp_path = "/tmp_load.sav";

    let script = format!(
        "try {{ \
        const data = localStorage.getItem('{}'); \
        if (data) {{ \
            const binary = atob(data); \
            const bytes = Uint8Array.from(binary, c => c.charCodeAt(0)); \
            FS.writeFile('{}', bytes); \
        }} \
    }} catch (e) {{ \
        console.error('Error loading from localStorage:', e); \
    }}",
        filename, tmp_path
    );

    match std::ffi::CString::new(script) {
        Ok(script) => unsafe {
            emscripten_run_script(script.as_ptr());
        },
        _ => eprintln!("Failed to create script string for loading from localStorage"),
    };

    let result = fs::read(tmp_path).ok();
    if result.is_some() {
        println!("Loaded save from localStorage with key: {}", filename);
        let _ = fs::remove_file(tmp_path);
    }
    result
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
