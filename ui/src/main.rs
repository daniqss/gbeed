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
    fs, io,
    path::{Path, PathBuf},
};

struct RaylibController {
    renderer: RaylibRenderer,
    serial_listener: RaylibSerialListener,
}

impl RaylibController {
    fn new() -> Self {
        Self {
            renderer: RaylibRenderer::new(),
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
                } else {
                    eprintln!("Error: Missing argument for --game");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-b" | "--boot" | "--boot_rom" => {
                if i + 1 < args.len() {
                    boot_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: Missing argument for --boot_rom");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            arg => {
                eprintln!("Unknown argument: {arg}");
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let (game_path, save_path) = match game_path {
        Some(path) => {
            let save_path = save_path_from_rom(&path);
            (path, save_path)
        }
        None => {
            print_help();
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No game ROM provided",
            )));
        }
    };

    let game_data = fs::read(&game_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to read game ROM at {game_path}: {e}"),
        )
    })?;

    // attempt to read the save file, if cartridge doesn't support saves it will discard it
    let save = match fs::read(&save_path) {
        Ok(data) => Some(data),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(Box::new(io::Error::other(format!(
                "Failed to read save file at {}: {e}",
                save_path.display()
            ))))
        }
    };

    let game = Cartridge::new(&game_data, save).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to create cartridge from ROM at {game_path}: {e}"),
        )
    })?;

    println!("{game}");

    let boot_rom = if let Some(ref path) = boot_path {
        Some(fs::read(path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to read boot ROM at {path}: {e}"),
            )
        })?)
    } else {
        None
    };

    let mut controller = RaylibController::new();

    let title = game.header.title.clone();
    let region = game.header.destination;
    controller.renderer.set_game_info(title, region);

    let mut gb = Dmg::new(game, boot_rom);

    while !controller.renderer.rl.window_should_close()
        && !controller.renderer.rl.is_key_down(KeyboardKey::KEY_ESCAPE)
    {
        input::update(&mut controller.renderer, &mut gb.joypad);

        gb.run(&mut controller)?;

        texture::update_tiles(&mut controller.renderer.tile_textures[0], gb.ppu.tile_block0());
        texture::update_tiles(&mut controller.renderer.tile_textures[1], gb.ppu.tile_block1());
        texture::update_tiles(&mut controller.renderer.tile_textures[2], gb.ppu.tile_block2());

        texture::update_bg_map(
            &mut controller.renderer.bg_map_texture,
            gb.ppu.bg_map0(),
            gb.ppu.tile_data(),
            gb.ppu.bg_tile_map_address(),
        );

        controller
            .renderer
            .update_bg_map(gb.ppu.bg_map0(), gb.ppu.tile_data());
    }

    if let Some(save_data) = gb.cartridge.save_game() {
        if let Err(e) = fs::write(&save_path, save_data) {
            eprintln!("Failed to write save file at {}: {e}", save_path.display());
        }
    }

    Ok(())
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
