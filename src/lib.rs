pub mod error;
pub mod prelude;
pub mod utils;

use crate::prelude::*;

pub fn run(file: Cartridge) -> Result<()> {
    println!("{:#?}", file);

    Ok(())
}
