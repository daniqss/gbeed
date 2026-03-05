use gbeed_core::prelude::*;

mod listener;
mod renderer;

use listener::RaylibSerialListener;
use renderer::RaylibRenderer;

use raylib::prelude::*;

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
        Some(path) => match std::fs::read(&path) {
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

    let boot_rom = if let Some(path) = boot_path {
        Some(std::fs::read(&path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read boot ROM at {path}: {e}"),
            )
        })?)
    } else {
        None
    };

    let renderer = Rc::new(RefCell::new(RaylibRenderer::new()));
    let serial_listener = Rc::new(RefCell::new(RaylibSerialListener));
    let mut gb = Dmg::new(game, boot_rom, Some(serial_listener), Some(renderer.clone()));

    // TODO: ugly code, needs refactor
    loop {
        let renderer_borrow = renderer.borrow_mut();
        if renderer_borrow.rl.window_should_close() {
            break;
        }

        if renderer_borrow.rl.is_key_down(KeyboardKey::KEY_ESCAPE) {
            break;
        }

        update_joypad(&renderer_borrow.rl, &mut gb.joypad);
        drop(renderer_borrow);

        gb.run()?;
    }

    Ok(())
}

fn update_joypad(rl: &RaylibHandle, joypad: &mut Joypad) {
    joypad.button_down(
        JoypadButton::Up,
        rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W),
    );
    joypad.button_down(
        JoypadButton::Down,
        rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S),
    );
    joypad.button_down(
        JoypadButton::Left,
        rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A),
    );
    joypad.button_down(
        JoypadButton::Right,
        rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D),
    );
    joypad.button_down(
        JoypadButton::A,
        rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_J),
    );
    joypad.button_down(
        JoypadButton::B,
        rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_K),
    );
    joypad.button_down(JoypadButton::Start, rl.is_key_down(KeyboardKey::KEY_L));
    joypad.button_down(
        JoypadButton::Select,
        rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_SEMICOLON),
    )
}

fn print_help() {
    println!("Usage: gbeed [OPTIONS]");
    println!("Options:");
    println!("  -g, --game <PATH>      Path to the game ROM file");
    println!("  -b, --boot <PATH>      Path to the boot ROM file (optional)");
    println!("  -h, --help             Print this help message");
}
