use gbeed::prelude::*;
use raylib::prelude::*;

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

    // Use game_rom and boot_room_data as needed

    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }

    Ok(())
    // gbeed::run(game_rom, boot_room_data)
}
