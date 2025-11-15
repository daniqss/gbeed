use gbeed::Cartridge;
use gbeed::prelude::*;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureQuery;
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

    let game_rom = std::fs::read(game_name)?;
    let _boot_room_data = std::fs::read(boot_room_name)?;

    let cartridge = Cartridge::new(&game_rom)?;

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init()?;

    let window = video_subsys
        .window(WINDOW_TITLE, SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // load our font
    let mut font = ttf_context.load_font(FONT_PATH, 32)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let line_spacing = font.recommended_line_spacing() as u32;
    let mut max_width = 0u32;

    let textures: Vec<Texture<'_>> = format!("{}", cartridge)
        .lines()
        .filter_map(|line| {
            let render_line = if line.is_empty() { " " } else { line };
            let surface = font
                .render(render_line)
                .blended(Color::RGBA(255, 255, 255, 200))
                .ok()?;
            let texture = texture_creator.create_texture_from_surface(&surface).ok()?;

            let TextureQuery { width, .. } = texture.query();

            if width > max_width {
                max_width = width;
            }

            Some(texture)
        })
        .collect();

    let total_height = line_spacing * (textures.len().max(1) as u32);
    let start_x = (SCREEN_WIDTH.saturating_sub(max_width)) / 2;
    let start_y = (SCREEN_HEIGHT.saturating_sub(total_height)) / 2;
    let mut current_y = start_y;

    for texture in textures {
        let TextureQuery { width, height, .. } = texture.query();

        let target_rect = Rect::new(start_x as i32, current_y as i32, width, height);
        canvas.copy(&texture, None, Some(target_rect))?;
        current_y += line_spacing;
    }

    canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
    }

    Ok(())
}
