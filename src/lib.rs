mod core;
pub mod error;
pub mod prelude;
pub mod utils;

use crate::prelude::*;
use core::{Cartridge, Dmg};

pub fn run(file: Vec<u8>) -> Result<()> {
    // parse cartridge data from raw ROM file
    let cartridge = match Cartridge::new(file) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    println!("{}", cartridge);
    cartridge.check_header_checksum()?;
    cartridge.check_global_checksum()?;
    let emulator = Dmg::new();

    Ok(())
}
