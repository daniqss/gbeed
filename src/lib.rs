mod core;
pub mod error;
pub mod prelude;
pub mod utils;

use crate::prelude::*;
use core::{Cartridge, Dmg};

pub fn run(game_rom: Vec<u8>, boot_rom: Vec<u8>) -> Result<()> {
    // parse cartridge data from game_rom ROM file
    let cartridge = match Cartridge::new(&game_rom) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    cartridge.check_header_checksum(&game_rom)?;

    let mut emulator = Dmg::new(cartridge, game_rom, boot_rom);

    emulator.run();

    Ok(())
}
