use gbeed::Cartridge;
use gbeed::prelude::*;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureQuery;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let game_name = args
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing game name"))?;
    let boot_room_name = args
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing boot room name"))?;

    let game_rom = std::fs::read(game_name)?;
    let boot_room_data = std::fs::read(boot_room_name)?;

    let cartridge = Cartridge::new(&game_rom)?;

    let sdl_context = sdl2::init().map_err(Error::Sdl2)?;
    let video_subsys = sdl_context.video().map_err(Error::Sdl2)?;
    let ttf_context = sdl2::ttf::init().map_err(Error::Sdl2)?;

    let window = video_subsys
        .window("SDL2_TTF Example", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| Error::Generic(e.to_string()))?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| Error::Sdl2(e.to_string()))?;
    let texture_creator = canvas.texture_creator();

    // Load a font
    let mut font = ttf_context
        .load_font("./assets/fonts/FreeSansBold.ttf", 32)
        .map_err(Error::Sdl2)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let line_spacing = font.recommended_line_spacing() as u32;
    let mut max_width = 0u32;

    let textures: Vec<Texture<'_>> = format!("{}", cartridge)
        .lines()
        .filter_map(|line| {
            let render_line = if line.is_empty() { " " } else { line };
            let surface = font
                .render(render_line)
                .blended(Color::RGBA(255, 0, 0, 255))
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

        let target_rect = rect!(start_x, current_y, width, height);
        canvas
            .copy(&texture, None, Some(target_rect))
            .map_err(|e| Error::Sdl2(e.to_string()))?;
        current_y += line_spacing;
    }

    canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump().map_err(Error::Sdl2)?.poll_iter() {
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
