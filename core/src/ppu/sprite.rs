use crate::prelude::*;

const PRIORITY: u8 = 0x80;
const YFLIP: u8 = 0x40;
const XFLIP: u8 = 0x20;
const PALETTE_NUMBER: u8 = 0x10;

pub const MAX_SPRITES_PER_LINE: u8 = 10;
pub const MAX_SPRITES_IN_OAM: u8 = 40;

/// Composed by one (normally) or two tiles.
/// Instead of create the struct to easily manipulate, for better optimization we could cast the OAM memory as a Sprite
#[derive(Debug, Default)]
pub struct Sprite {
    pub xpos: u8,
    pub ypos: u8,
    pub tile_index: u8,
    flags: u8,
}

impl Sprite {
    pub fn from_oam(sprite_data: &[u8]) -> Self {
        Self {
            ypos: sprite_data[0].wrapping_sub(16),
            xpos: sprite_data[1].wrapping_sub(8),
            tile_index: sprite_data[2],
            flags: sprite_data[3],
        }
    }

    pub fn _to_oam(&self, sprite_data: &mut [u8]) {
        sprite_data[0] = self.ypos.wrapping_add(16);
        sprite_data[1] = self.xpos.wrapping_add(8);
        sprite_data[2] = self.tile_index;
        sprite_data[3] = self.flags;
    }

    bit_accessors! {
        target: flags;

        PRIORITY,
        YFLIP,
        XFLIP,
        PALETTE_NUMBER
    }
}
