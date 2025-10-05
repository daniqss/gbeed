mod core;
pub mod error;
pub mod prelude;
pub mod utils;

use crate::prelude::*;
use core::*;

pub fn run(file: Cartridge) -> Result<()> {
    println!("{:#?}", file);

    let emulator = Dmg::new();

    Ok(())
}
