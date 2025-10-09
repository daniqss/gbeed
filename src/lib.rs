mod core;
pub mod error;
pub mod prelude;
pub mod utils;

use crate::prelude::*;
use core::{Cartridge, Dmg};

pub fn run(raw: Vec<u8>) -> Result<()> {
    // parse cartridge data from raw ROM file
    let cartridge = match Cartridge::new(&raw) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    println!("{}", cartridge);
    cartridge.check_header_checksum(&raw)?;

    let mut emulator = Dmg::new(cartridge, raw);

    emulator.run();

    Ok(())
}
