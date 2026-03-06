use gbeed_core::prelude::*;

mod listener;
mod renderer;

use listener::RaylibSerialListener;
use renderer::{ButtonStates, RaylibRenderer};

use raylib::prelude::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // ── Argument parsing ──────────────────────────────────────────────────
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

    // ── Load cartridge ────────────────────────────────────────────────────
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

    // ── Build renderer and populate static game info ──────────────────────
    let renderer = Rc::new(RefCell::new(RaylibRenderer::new()));

    {
        let mut r = renderer.borrow_mut();

        // TODO: adapt these calls to match whatever Cartridge exposes.
        //
        //   Common patterns in GB emulator cores:
        //     game.title()          → &str
        //     game.name.clone()     → String
        //     game.header.title     → [u8; N]  (needs from_utf8 / lossy)
        //
        //   For region / destination code:
        //     game.destination_code() → u8  (0x00 = Japan, 0x01 = Overseas)
        //     game.region()           → &str
        //
        //   Fallback: if the info is unavailable just pass "N/A".

        let title = game.header.title.clone();
        let region = game.header.destination;
        r.set_game_info(title, region);
    }

    // ── Assemble emulator ─────────────────────────────────────────────────
    let serial_listener = Rc::new(RefCell::new(RaylibSerialListener));
    let mut gb = Dmg::new(game, boot_rom, Some(serial_listener), Some(renderer.clone()));

    // ── Main loop ─────────────────────────────────────────────────────────
    loop {
        // 1. Check exit conditions (borrow briefly, then release)
        {
            let r = renderer.borrow();
            if r.rl.window_should_close() || r.rl.is_key_down(KeyboardKey::KEY_ESCAPE) {
                break;
            }
        }

        // 2. Read all raw input states from raylib (immutable borrow)
        let input = {
            let r = renderer.borrow();
            read_input(&r.rl)
        };

        // 3. Forward input to the emulator joypad
        apply_joypad(&input, &mut gb.joypad);

        // 4. Mirror input into the renderer's button state (for the UI display)
        {
            let mut r = renderer.borrow_mut();
            r.buttons = input;
        }

        // 5. Check whether the FPS button was clicked this frame
        if renderer.borrow().fps_btn_clicked() {
            renderer.borrow_mut().cycle_fps();
        }

        // 6. Run one video-frame worth of emulation
        //    This will eventually call renderer.draw_screen() when the PPU
        //    signals a VBlank.
        gb.run()?;

        // 7. Update tile viewers from live VRAM
        //
        //    TODO: replace `gb.read_memory(addr)` with however your Dmg
        //    exposes bus reads.  Typical alternatives:
        //
        //      gb.bus.read(addr: u16) -> u8
        //      gb.mmu.read_byte(addr)
        //      gb.ppu.vram[addr as usize - 0x8000]  (direct slice)
        //
        //    If your core exposes a contiguous VRAM slice you can pass it
        //    directly instead of building a Vec:
        //
        //      let vram = gb.ppu.vram.as_slice();   // &[u8; 0x2000]
        //      r.update_tiles(0, &vram[0x0000..0x0800]);
        //      r.update_tiles(1, &vram[0x0800..0x1000]);
        //      r.update_tiles(2, &vram[0x1000..0x1800]);

        {
            let mut r = renderer.borrow_mut();

            // Read the three 2 KiB VRAM blocks in one pass
            let vram: Vec<u8> = (0x8000_u16..=0x97FF_u16).map(|addr| gb.read(addr)).collect();

            r.update_tiles(0, &vram[0x0000..0x0800]); // $8000-$87FF
            r.update_tiles(1, &vram[0x0800..0x1000]); // $8800-$8FFF
            r.update_tiles(2, &vram[0x1000..0x1800]); // $9000-$97FF
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
//  Input helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot of every button at the current frame.
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

/// Apply a button-state snapshot to the emulator joypad.
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

// ─────────────────────────────────────────────────────────────────────────────
fn print_help() {
    println!("Usage: gbeed [OPTIONS]");
    println!("Options:");
    println!("  -g, --game <PATH>      Path to the game ROM file");
    println!("  -b, --boot <PATH>      Path to the boot ROM file (optional)");
    println!("  -h, --help             Print this help message");
}
