use gbeed_core::{prelude::*, Controller, Renderer, SerialListener};

mod colors;
mod listener;
mod renderer;

use listener::RaylibSerialListener;
use renderer::{ButtonStates, RaylibRenderer};

use raylib::prelude::*;

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

    let game: Cartridge = match game_path {
        Some(ref path) => match std::fs::read(path) {
            Ok(data) => Cartridge::new(data),
            Err(e) => {
                print_help();
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Failed to read game ROM at {path}: {e}"),
                )));
            }
        },
        None => {
            print_help();
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No game ROM provided",
            )));
        }
    };

    let boot_rom = if let Some(ref path) = boot_path {
        Some(std::fs::read(path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
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

    while controller.renderer.rl.window_should_close()
        || controller.renderer.rl.is_key_down(KeyboardKey::KEY_ESCAPE)
    {
        let input = read_input(&controller.renderer.rl);

        apply_joypad(&input, &mut gb.joypad);

        controller.renderer.buttons = input;

        if controller.renderer.fps_btn_clicked() {
            controller.renderer.cycle_fps();
        }

        gb.run(&mut controller)?;

        let vram: Vec<u8> = (0x8000_u16..=0x9BFF_u16).map(|addr| gb.read(addr)).collect();

        controller.renderer.update_tiles(0, &vram[0x0000..0x0800]);
        controller.renderer.update_tiles(1, &vram[0x0800..0x1000]);
        controller.renderer.update_tiles(2, &vram[0x1000..0x1800]);
        controller
            .renderer
            .update_bg_map(&vram[0x1800..0x1C00], &vram[0x0000..0x1000]);
    }

    Ok(())
}

fn read_input(rl: &RaylibHandle) -> ButtonStates {
    ButtonStates {
        up: rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
        down: rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S),
        left: rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
        right: rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
        a: rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_J),
        b: rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_K),
        start: rl.is_key_down(KeyboardKey::KEY_L),
        select: rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_SEMICOLON),
    }
}

/// Apply a button-state to the emulator joypad.
fn apply_joypad(s: &ButtonStates, joypad: &mut Joypad) {
    joypad.button_down(JoypadButton::Up, s.up);
    joypad.button_down(JoypadButton::Down, s.down);
    joypad.button_down(JoypadButton::Left, s.left);
    joypad.button_down(JoypadButton::Right, s.right);
    joypad.button_down(JoypadButton::A, s.a);
    joypad.button_down(JoypadButton::B, s.b);
    joypad.button_down(JoypadButton::Start, s.start);
    joypad.button_down(JoypadButton::Select, s.select);
}

fn print_help() {
    println!("Usage: gbeed [OPTIONS]");
    println!("Options:");
    println!("  -g, --game <PATH>      Path to the game ROM file");
    println!("  -b, --boot <PATH>      Path to the boot ROM file (optional)");
    println!("  -h, --help             Print this help message");
}
