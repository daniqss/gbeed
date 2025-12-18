use gbeed::Cartridge;
use gbeed::Dmg;
use gbeed::Joypad;
use gbeed::prelude::*;

use raylib::prelude::*;

use std::io::{self, ErrorKind};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const FONT_PATH: &str = "./assets/fonts/CaskaydiaCoveNerdFont-BoldItalic.ttf";
const WINDOW_TITLE: &str = "gbeed";

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

    // let mut dmg = Dmg::new(Some(game), Some(boot_room_data));

    let (w, h) = (1920, 1080);
    let (mut rl, thread) = raylib::init()
        .size(w, h)
        .title("gbeed -- desktop")
        .resizable()
        .build();
    let black = Color::new(0, 0, 0, 255);
    let ray_white = Color::new(255, 255, 255, 255);

    while !rl.window_should_close() {
        rl.draw(&thread, |mut d| {
            d.clear_background(ray_white);
            for (i, line) in format!("{}", game).lines().enumerate() {
                d.draw_text(line, 10, 10 + i as i32 * 20, 20, black);
            }
        });
    }

    Ok(())
}

// fn update_joypad(is_down: bool, key: Keycode, jp: &mut Joypad) {
//     match key {
//         Keycode::W => {
//             jp.set_select_directions(is_down);
//             jp.set_input_up_select(is_down)
//         }
//         Keycode::S => {
//             jp.set_select_directions(is_down);
//             jp.set_input_down_start(is_down)
//         }
//         Keycode::A => {
//             jp.set_select_directions(is_down);
//             jp.set_input_left_b(is_down)
//         }
//         Keycode::D => {
//             jp.set_select_directions(is_down);
//             jp.set_input_right_a(is_down)
//         }
//         // a
//         Keycode::J => {
//             jp.set_select_buttons(is_down);
//             jp.set_input_right_a(is_down)
//         }
//         // b
//         Keycode::X => {
//             jp.set_select_buttons(is_down);
//             jp.set_input_left_b(is_down)
//         }
//         // start
//         Keycode::Return => {
//             jp.set_select_buttons(is_down);
//             jp.set_input_down_start(is_down)
//         }
//         // select
//         Keycode::RShift => {
//             jp.set_select_buttons(is_down);
//             jp.set_input_up_select(is_down)
//         }
//         _ => {}
//     }
// }
