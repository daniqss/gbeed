use crate::{Dmg, prelude::*};

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
    // we should probably implement slices for mmu to have better access
    pub fn from_oam(gb: &Dmg, index: u16) -> Self {
        Self {
            ypos: gb[index].wrapping_sub(16),
            xpos: gb[index + 1].wrapping_sub(8),
            tile_index: gb[index + 2],
            flags: gb[index + 3],
        }
    }

    pub fn to_oam(&self, gb: &mut Dmg, index: u16) {
        gb[index] = self.ypos;
        gb[index + 1] = self.xpos;
        gb[index + 2] = self.tile_index;
        gb[index + 3] = self.flags;
    }

    bit_accessors! {
        target: flags;

        PRIORITY,
        YFLIP,
        XFLIP,
        PALETTE_NUMBER
    }
}
