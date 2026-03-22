use gbeed_core::{prelude::*, Controller, Renderer, SerialListener};

mod colors;
mod input;
mod listener;
mod renderer;
mod texture;

use listener::RaylibSerialListener;
use renderer::RaylibRenderer;

use raylib::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

struct RaylibController {
    renderer: RaylibRenderer,
    serial_listener: RaylibSerialListener,
}

impl RaylibController {
    fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
        Self {
            renderer: RaylibRenderer::new(rl, thread),
            serial_listener: RaylibSerialListener,
        }
    }
}

impl Renderer for RaylibController {
    fn read_pixel(&self, x: usize, y: usize) -> u32 { self.renderer.read_pixel(x, y) }
    fn write_pixel(&mut self, x: usize, y: usize, color: u32) { self.renderer.write_pixel(x, y, color); }
    fn get_color(&self, palette: u8, color_id: u8) -> u32 { self.renderer.get_color(palette, color_id) }
    fn draw_screen(&mut self) { self.renderer.draw_screen() }
}

impl SerialListener for RaylibController {
    fn on_transfer(&mut self, data: u8) { self.serial_listener.on_transfer(data) }
}

impl Controller for RaylibController {}

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
        app.save_game();
    }

    #[cfg(target_arch = "wasm32")]
    unsafe {
        let app_ptr = Box::into_raw(Box::new(app));
        emscripten_set_main_loop_arg(wasm_main_loop, app_ptr, 0, 1);
    }

    Ok(())
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
unsafe fn emscripten_sync_fs() {
    let script =
        std::ffi::CString::new("FS.syncfs(false, function (err) { if (err) console.error(err); });").unwrap();
    emscripten_run_script(script.as_ptr());
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" fn wasm_main_loop(app: *mut EmulatorApp) {
    let app = &mut *app;
    if let Err(e) = app.update() {
        eprintln!("Error during update: {e}");
    }
    // probably the best choice will be to add a save button
}

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
        let save_path = save_path_from_rom(path);

        // On WASM, we might want to use a fixed name or raylib storage,
        // but for now we'll stick to the path-based logic which works with Emscripten's MEMFS/IDBFS
        let save = fs::read(&save_path).ok();

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

            texture::update_tiles(
                &mut self.controller.renderer.tile_textures[0],
                gb.ppu.tile_block0(),
            );
            texture::update_tiles(
                &mut self.controller.renderer.tile_textures[1],
                gb.ppu.tile_block1(),
            );
            texture::update_tiles(
                &mut self.controller.renderer.tile_textures[2],
                gb.ppu.tile_block2(),
            );

            texture::update_bg_map(
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
            d.clear_background(colors::BACKGROUND);
            let msg = "Drag and Drop a Game Boy ROM to start";
            let font_size = 20;
            let width = d.measure_text(msg, font_size);
            d.draw_text(
                msg,
                (d.get_screen_width() - width) / 2,
                (d.get_screen_height() - font_size) / 2,
                font_size,
                colors::FOREGROUND,
            );
        }
        Ok(())
    }

    fn save_game(&mut self) {
        if let Some(ref gb) = self.gb {
            if let Some(save_data) = gb.cartridge.save_game() {
                if let Some(ref save_path) = self.save_path {
                    if let Err(e) = fs::write(save_path, save_data) {
                        eprintln!("Failed to write save file at {}: {e}", save_path.display());
                    } else {
                        #[cfg(target_arch = "wasm32")]
                        unsafe {
                            // Sync Emscripten filesystem to IndexedDB if possible
                            emscripten_sync_fs();
                        }
                    }
                }
            }
        }
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
