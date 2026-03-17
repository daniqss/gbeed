use crate::cartrigde::{CartridgeError, CartridgeResult};

/// # Ram Size
/// Defines how much RAM is provided by the cartridge
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RamSize {
    #[default]
    None,
    Unused,
    Ram8KB,
    Ram32KB,
    Ram128KB,
    Ram64KB,
}

impl RamSize {
    pub fn new(value: u8) -> CartridgeResult<Self> {
        match value {
            0x00 => Ok(RamSize::None),
            0x01 => Err(CartridgeError::InvalidRamSize(
                Some(RamSize::Unused),
                "Unused RAM size",
            )),
            0x02 => Ok(RamSize::Ram8KB),
            0x03 => Ok(RamSize::Ram32KB),
            0x04 => Ok(RamSize::Ram128KB),
            0x05 => Ok(RamSize::Ram64KB),
            _ => Err(CartridgeError::InvalidRamSize(None, "Unknown RAM size")),
        }
    }

    pub fn get_banks_count(&self) -> Option<u16> {
        match self {
            RamSize::None | RamSize::Unused => None,
            RamSize::Ram8KB => Some(1),
            RamSize::Ram32KB => Some(4),
            RamSize::Ram128KB => Some(16),
            RamSize::Ram64KB => Some(8),
        }
    }

    pub fn get_size(&self) -> u32 {
        match self {
            RamSize::None | RamSize::Unused => 0,
            RamSize::Ram8KB => 8 * 1024,
            RamSize::Ram32KB => 32 * 1024,
            RamSize::Ram128KB => 128 * 1024,
            RamSize::Ram64KB => 64 * 1024,
        }
    }
}
