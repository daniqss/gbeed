use gbeed_core::prelude::*;

mod colors;
mod listener;
mod renderer;

use listener::RaylibSerialListener;
use renderer::{ButtonStates, RaylibRenderer};

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

    let renderer = Rc::new(RefCell::new(RaylibRenderer::new()));

    {
        let mut r = renderer.borrow_mut();

        let title = game.header.title.clone();
        let region = game.header.destination;
        r.set_game_info(title, region);
    }

    let serial_listener = Rc::new(RefCell::new(RaylibSerialListener));
    let mut gb = Dmg::new(game, boot_rom, Some(serial_listener), Some(renderer.clone()));

    loop {
        {
            let r = renderer.borrow();
            if r.rl.window_should_close() || r.rl.is_key_down(KeyboardKey::KEY_ESCAPE) {
                break;
            }
        }

        let input = {
            let r = renderer.borrow();
            read_input(&r.rl)
        };

        apply_joypad(&input, &mut gb.joypad);

        {
            let mut r = renderer.borrow_mut();
            r.buttons = input;
        }

        if renderer.borrow().fps_btn_clicked() {
            renderer.borrow_mut().cycle_fps();
        }

        gb.run()?;

        {
            let mut r = renderer.borrow_mut();

            let vram: Vec<u8> = (0x8000_u16..=0x97FF_u16).map(|addr| gb.read(addr)).collect();

            r.update_tiles(0, &vram[0x0000..0x0800]);
            r.update_tiles(1, &vram[0x0800..0x1000]);
            r.update_tiles(2, &vram[0x1000..0x1800]);
        }
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
