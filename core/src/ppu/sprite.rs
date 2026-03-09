use crate::{prelude::*, Ppu};

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
    pub fn from_oam(ppu: &Ppu, index: u16) -> Self {
        Self {
            ypos: ppu.read(index).wrapping_sub(16),
            xpos: ppu.read(index + 1).wrapping_sub(8),
            tile_index: ppu.read(index + 2),
            flags: ppu.read(index + 3),
        }
    }

    pub fn _to_oam(&self, ppu: &mut Ppu, index: u16) {
        ppu.write(index, self.ypos);
        ppu.write(index + 1, self.xpos);
        ppu.write(index + 2, self.tile_index);
        ppu.write(index + 3, self.flags);
    }

    bit_accessors! {
        target: flags;

        PRIORITY,
        YFLIP,
        XFLIP,
        PALETTE_NUMBER
    }
}
