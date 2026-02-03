use gbeed::Cartridge;
use gbeed::Dmg;
use gbeed::Joypad;
use gbeed::core::ppu::DMG_SCREEN_HEIGHT;
use gbeed::core::ppu::DMG_SCREEN_WIDTH;
use gbeed::prelude::*;

use raylib::ffi::PixelFormat;
use raylib::prelude::*;

// we should distinguish between desktop arm and armv6 32 bits of the raspberry pi zero
#[cfg(target_arch = "arm")]
const SCREEN_WIDTH: i32 = 400;
#[cfg(not(target_arch = "arm"))]
const SCREEN_WIDTH: i32 = 1920;
#[cfg(target_arch = "arm")]
const SCREEN_HEIGHT: i32 = 240;
#[cfg(not(target_arch = "arm"))]
const SCREEN_HEIGHT: i32 = 1080;
#[cfg(target_arch = "arm")]
const WINDOW_TITLE: &str = "gbeed";
#[cfg(not(target_arch = "arm"))]
const WINDOW_TITLE: &str = "gbeed -- desktop";

fn main() -> Result<()> {
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
                eprintln!("Unknown argument: {}", arg);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let game = if let Some(path) = game_path {
        let data = std::fs::read(&path)
            .map_err(|e| Error::Generic(format!("Failed to read game ROM at {}: {}", path, e)))?;
        Some(Cartridge::new(data)?)
    } else {
        None
    };

    let boot_rom = if let Some(path) = boot_path {
        Some(
            std::fs::read(&path)
                .map_err(|e| Error::Generic(format!("Failed to read boot ROM at {}: {}", path, e)))?,
        )
    } else {
        None
    };

    let mut gb = Dmg::new(game, boot_rom);

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title(WINDOW_TITLE)
        .resizable()
        .build();
    rl.set_target_fps(60);

    let mut frame_image =
        Image::gen_image_color(DMG_SCREEN_WIDTH as i32, DMG_SCREEN_HEIGHT as i32, Color::BLACK);
    frame_image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
    let mut frame = rl
        .load_texture_from_image(&thread, &frame_image)
        .expect("Failed to load texture");

    while !rl.window_should_close() {
        gb.run()?;

        draw_screen(&mut rl, &thread, &mut gb, &mut frame);
        update_joypad(&mut gb.joypad, rl.get_key_pressed());
    }

    Ok(())
}

fn _draw_cartridge(rl: &mut RaylibHandle, thread: &RaylibThread, gb: &Dmg) {
    let white = Color::new(255, 255, 255, 255);

    rl.draw(&thread, |mut d| {
        // d.clear_background(ray_white);

        if let Some(game) = &gb.bus.game {
            for (i, line) in format!("{}", game).lines().enumerate() {
                d.draw_text(line, 10, 10 + i as i32 * 20, 20, white);
            }
        }
    });
}

fn draw_screen(rl: &mut RaylibHandle, thread: &RaylibThread, gb: &mut Dmg, texture: &mut Texture2D) {
    let mut pixels = Vec::with_capacity(DMG_SCREEN_WIDTH * DMG_SCREEN_HEIGHT * 4);

    for y in 0..DMG_SCREEN_HEIGHT {
        for x in 0..DMG_SCREEN_WIDTH {
            let color = gb.ppu.framebuffer[y][x];
            pixels.push(((color >> 16) & 0xFF) as u8);
            pixels.push(((color >> 8) & 0xFF) as u8);
            pixels.push((color & 0xFF) as u8);
        }
    }

    let _ = texture.update_texture(&pixels);

    rl.draw(&thread, |mut d| {
        d.clear_background(Color::BLACK);

        let screen_w = d.get_screen_width() as f32;
        let screen_h = d.get_screen_height() as f32;
        let scale = (screen_w / DMG_SCREEN_WIDTH as f32).min(screen_h / DMG_SCREEN_HEIGHT as f32);

        let dest_w = DMG_SCREEN_WIDTH as f32 * scale;
        let dest_h = DMG_SCREEN_HEIGHT as f32 * scale;
        let dest_x = (screen_w - dest_w) / 2.0;
        let dest_y = (screen_h - dest_h) / 2.0;

        d.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, DMG_SCREEN_WIDTH as f32, DMG_SCREEN_HEIGHT as f32),
            Rectangle::new(dest_x, dest_y, dest_w, dest_h),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        d.draw_fps(10, screen_h as i32 - 20);
    });

    // draw_cartridge(rl, thread, gb);
}

fn update_joypad(jp: &mut Joypad, key: Option<KeyboardKey>) {
    match key {
        Some(KeyboardKey::KEY_UP | KeyboardKey::KEY_W) => {
            jp.set_select_directions(true);
            jp.set_input_up_select(true)
        }
        Some(KeyboardKey::KEY_DOWN | KeyboardKey::KEY_S) => {
            jp.set_select_directions(true);
            jp.set_input_down_start(true)
        }
        Some(KeyboardKey::KEY_LEFT | KeyboardKey::KEY_A) => {
            jp.set_select_directions(true);
            jp.set_input_left_b(true)
        }
        Some(KeyboardKey::KEY_RIGHT | KeyboardKey::KEY_D) => {
            jp.set_select_directions(true);
            jp.set_input_right_a(true)
        }
        Some(KeyboardKey::KEY_J) => {
            jp.set_select_buttons(true);
            jp.set_input_right_a(true)
        }
        Some(KeyboardKey::KEY_X) => {
            jp.set_select_buttons(true);
            jp.set_input_left_b(true)
        }
        Some(KeyboardKey::KEY_ENTER) => {
            jp.set_select_buttons(true);
            jp.set_input_down_start(true)
        }
        Some(KeyboardKey::KEY_RIGHT_SHIFT) | Some(KeyboardKey::KEY_LEFT_SHIFT) => {
            jp.set_select_buttons(true);
            jp.set_input_up_select(true)
        }
        _ => {}
    }
}

fn print_help() {
    println!("Usage: gbeed [OPTIONS]");
    println!("Options:");
    println!("  -g, --game <PATH>      Path to the game ROM file");
    println!("  -b, --boot <PATH>      Path to the boot ROM file (optional)");
    println!("  -h, --help             Print this help message");
}
