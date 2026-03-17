mod features;
mod header;
mod mbc;

use crate::{
    EXTERNAL_RAM_END, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    prelude::*,
};

use features::CartridgeFeatures;
use header::CartridgeHeader;
pub use header::{RamSize, RomSize};
use mbc::{CartridgeType, MemoryBankController, select_mbc};

/// Used for MBC1M multicart cartridge detection
/// Used in several emulators for this purpose, considered fair use of the cartridge data
pub const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00,
    0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB,
    0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];
mem_range!(CARTRIDGE_LOGO, 0x0104, 0x0104 + NINTENDO_LOGO.len() as u16 - 1);

#[derive(Debug)]
pub enum CartridgeError {
    InvalidRomSize(Option<RomSize>, &'static str),
    InvalidRamSize(Option<RamSize>, &'static str),
    InvalidMBCRomRamCombination(CartridgeType, RomSize, RamSize, &'static str),
    UnsupportedCartridgeType(CartridgeType),
    IncorrectHeaderChecksum(u8, u8),
    IncorrectGlobalChecksum(u16, u16),
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::InvalidRomSize(size, message) => write!(
                f,
                "Invalid ROM size: {}. {}",
                size.map_or("Unknown".to_string(), |s| format!("{:?}", s)),
                message
            ),
            CartridgeError::InvalidRamSize(size, message) => write!(
                f,
                "Invalid RAM size: {}. {}",
                size.map_or("Unknown".to_string(), |s| format!("{:?}", s)),
                message
            ),
            CartridgeError::UnsupportedCartridgeType(cartridge_type) => {
                write!(f, "Unsupported cartridge type: {:?}", cartridge_type)
            }
            CartridgeError::InvalidMBCRomRamCombination(cartridge_type, rom_size, ram_size, message) => {
                writeln!(
                    f,
                    "Invalid ROM/RAM combination for cartridge type {:?}: ROM size: {:?}, RAM size: {:?}.",
                    cartridge_type, rom_size, ram_size,
                )?;
                writeln!(f, "{}", message)
            }
            CartridgeError::IncorrectHeaderChecksum(calculated, expected) => write!(
                f,
                "Incorrect header checksum: calculated 0x{calculated:02X}, expected 0x{expected:02X}"
            ),
            CartridgeError::IncorrectGlobalChecksum(calculated, expected) => write!(
                f,
                "Incorrect global checksum: calculated 0x{calculated:04X}, expected 0x{expected:04X}"
            ),
        }
    }
}

pub type CartridgeResult<T> = std::result::Result<T, CartridgeError>;

pub struct Cartridge {
    pub header: CartridgeHeader,
    pub features: CartridgeFeatures,
    mbc: Box<dyn MemoryBankController>,
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge").field("header", &self.header).finish()
    }
}

impl Default for Cartridge {
    fn default() -> Self { Cartridge::new(&[0; (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize], None).unwrap() }
}

impl Cartridge {
    pub fn new(raw_rom: &[u8], save: Option<Vec<u8>>) -> CartridgeResult<Self> {
        let header = CartridgeHeader::new(raw_rom)?;

        let features = CartridgeFeatures::new(&header.cartridge_type);
        if !features.supports_saves() && save.is_some() {
            return Err(CartridgeError::InvalidMBCRomRamCombination(
                header.cartridge_type,
                header.rom_size,
                header.ram_size,
                "Cartridge does not support saves, but save data was provided",
            ));
        }

        let mbc = select_mbc(raw_rom, save, &features, &header)?;

        Ok(Self {
            header,
            features,
            mbc,
        })
    }

    #[inline(always)]
    pub fn supports_saves(&self) -> bool { self.features.supports_saves() }

    #[inline(always)]
    pub fn save_game(&self) -> Option<&[u8]> {
        if self.supports_saves() {
            self.mbc.get_ram()
        } else {
            None
        }
    }

    /// Used to not need to check if the read/write is for the boot ROM or the cartridge ROM in the MBCs
    pub fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) { self.mbc.swap_boot_rom(boot_rom); }

    /// # Header checksum
    /// Checked by real hardware by the boot ROM
    pub fn check_header_checksum(&self, raw_rom: &[u8]) -> CartridgeResult<()> {
        let header_sum = raw_rom[header::TITLE_START as usize..=header::GAME_VERSION]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));

        match header_sum == self.header.header_checksum {
            true => Ok(()),
            false => Err(CartridgeError::IncorrectHeaderChecksum(
                header_sum,
                self.header.header_checksum,
            )),
        }
    }

    /// # Global checksum
    /// Not actually checked by real hardware
    /// We'll use in Cartridge creation for now to verify correct file parsing and integrity
    pub fn check_global_checksum(&self, raw_rom: &[u8]) -> CartridgeResult<()> {
        let cartridge_sum: u16 = raw_rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
            match i != header::GLOBAL_CHECKSUM_START as usize && i != header::GLOBAL_CHECKSUM_END as usize {
                true => acc.wrapping_add(b as u16),
                false => acc,
            }
        });

        match cartridge_sum == self.header.global_checksum {
            true => Ok(()),
            false => Err(CartridgeError::IncorrectGlobalChecksum(
                cartridge_sum,
                self.header.global_checksum,
            )),
        }
    }
}

impl Accessible<u16> for Cartridge {
    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.mbc.read_rom(address),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.mbc.read_ram(address),

            _ => unreachable!(
                "Cartrigde: read of address {address:04X} should have been handled by other components"
            ),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => self.mbc.write_rom(address, value),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => self.mbc.write_ram(address, value),

            _ => unreachable!(
                "Cartrigde: write of address {address:04X} should have been handled by other components"
            ),
        }
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { writeln!(f, "{}", self.header) }
}
