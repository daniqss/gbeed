use crate::cartrigde::{CartridgeError, CartridgeResult};

/// # Rom Size
/// Other formats are listed in unofficial docs, but they're not found in real cartridges
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RomSize {
    #[default]
    Rom32KB = 0x00,
    Rom64KB = 0x01,
    Rom128KB = 0x02,
    Rom256KB = 0x03,
    Rom512KB = 0x04,
    Rom1MB = 0x05,
    Rom2MB = 0x06,
    Rom4MB = 0x07,
    Rom8MB = 0x08,
    Rom1_1MB = 0x52,
    Rom1_2MB = 0x53,
    Rom1_5MB = 0x54,
}

impl RomSize {
    pub fn new(value: u8) -> CartridgeResult<Self> {
        match value {
            0x00 => Ok(RomSize::Rom32KB),
            0x01 => Ok(RomSize::Rom64KB),
            0x02 => Ok(RomSize::Rom128KB),
            0x03 => Ok(RomSize::Rom256KB),
            0x04 => Ok(RomSize::Rom512KB),
            0x05 => Ok(RomSize::Rom1MB),
            0x06 => Ok(RomSize::Rom2MB),
            0x07 => Ok(RomSize::Rom4MB),
            0x08 => Ok(RomSize::Rom8MB),
            0x52 => Err(CartridgeError::InvalidRomSize(
                Some(RomSize::Rom1_1MB),
                "Unofficial ROM size -> 1.1MB",
            )),
            0x53 => Err(CartridgeError::InvalidRomSize(
                Some(RomSize::Rom1_2MB),
                "Unofficial ROM size -> 1.2MB",
            )),
            0x54 => Err(CartridgeError::InvalidRomSize(
                Some(RomSize::Rom1_5MB),
                "Unofficial ROM size -> 1.5MB",
            )),
            _ => Err(CartridgeError::InvalidRomSize(None, "Unknown ROM size")),
        }
    }

    pub fn get_banks_count(&self) -> u16 {
        match self {
            RomSize::Rom32KB => 2,
            RomSize::Rom64KB => 4,
            RomSize::Rom128KB => 8,
            RomSize::Rom256KB => 16,
            RomSize::Rom512KB => 32,
            RomSize::Rom1MB => 64,
            RomSize::Rom2MB => 128,
            RomSize::Rom4MB => 256,
            RomSize::Rom8MB => 512,
            RomSize::Rom1_1MB => 72,
            RomSize::Rom1_2MB => 80,
            RomSize::Rom1_5MB => 96,
        }
    }

    pub fn get_size(&self) -> u32 {
        match self {
            RomSize::Rom32KB => 32 * 1024,
            RomSize::Rom64KB => 64 * 1024,
            RomSize::Rom128KB => 128 * 1024,
            RomSize::Rom256KB => 256 * 1024,
            RomSize::Rom512KB => 512 * 1024,
            RomSize::Rom1MB => 1024 * 1024,
            RomSize::Rom2MB => 2 * 1024 * 1024,
            RomSize::Rom4MB => 4 * 1024 * 1024,
            RomSize::Rom8MB => 8 * 1024 * 1024,
            RomSize::Rom1_1MB => 72 * 16 * 1024,
            RomSize::Rom1_2MB => 80 * 16 * 1024,
            RomSize::Rom1_5MB => 96 * 16 * 1024,
        }
    }
}
