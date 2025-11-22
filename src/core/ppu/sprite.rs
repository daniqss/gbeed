use crate::prelude::*;

const PRIORITY: u8 = 0x80;
const YFLIP: u8 = 0x40;
const XFLIP: u8 = 0x20;
const PALETTE_NUMBER: u8 = 0x10;

#[derive(Debug, Default)]
pub struct Sprite {
    pub xpos: u8,
    pub ypos: u8,
    pub tile_index: u8,
    flags: u8,
}

impl Sprite {
    bit_accessors! {
        target: flags;

        PRIORITY,
        YFLIP,
        XFLIP,
        PALETTE_NUMBER
    }
}
