use gbeed::prelude::*;

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
    gbeed::run(game_rom, boot_room_data)
}
