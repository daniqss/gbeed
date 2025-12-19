use gbeed::Cartridge;
use gbeed::Dmg;
use gbeed::Joypad;
use gbeed::prelude::*;

use raylib::prelude::*;

use std::io::{self, ErrorKind};

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
    let mut args = std::env::args().skip(1);

    let game_name = args
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Missing game name"))?;
    let boot_room_name = args
        .next()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Missing boot room name"))?;

    let game = Cartridge::new(std::fs::read(game_name)?)?;
    let boot_room_data = std::fs::read(boot_room_name)?;
    let mut gb = Dmg::new(Some(game), Some(boot_room_data));

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title(WINDOW_TITLE)
        .resizable()
        .build();
    let black = Color::new(0, 0, 0, 255);
    let ray_white = Color::new(255, 255, 255, 255);

    while !rl.window_should_close() {
        rl.draw(&thread, |mut d| {
            d.clear_background(ray_white);

            if let Some(game) = &gb.bus.game {
                for (i, line) in format!("{}", game).lines().enumerate() {
                    d.draw_text(line, 10, 10 + i as i32 * 20, 20, black);
                }
            }
        });
        update_joypad(&mut gb.joypad, rl.get_key_pressed());
    }

    Ok(())
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
