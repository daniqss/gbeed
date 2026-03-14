mod header;
mod mbc;

use crate::{
    EXTERNAL_RAM_END, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    prelude::*,
};

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

pub enum CartridgeError {
    InvalidRomSize(Option<RomSize>, &'static str),
    InvalidRamSize(Option<RamSize>, &'static str),
    InvalidMBCRomRamCombination(CartridgeType, RomSize, RamSize, &'static str),
    UnsupportedCartridgeType(CartridgeType),
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
        }
    }
}

pub type CartridgeResult<T> = std::result::Result<T, CartridgeError>;

pub struct Cartridge {
    pub header: CartridgeHeader,
    pub mbc: Box<dyn MemoryBankController>,
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge").field("header", &self.header).finish()
    }
}

impl Default for Cartridge {
    fn default() -> Self { Cartridge::new(&[0; (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize]).unwrap() }
}

impl Cartridge {
    pub fn new(raw_rom: &[u8]) -> Result<Self> {
        let header = match CartridgeHeader::new(raw_rom) {
            Ok(header) => header,
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse cartridge header: {e}"),
                )));
            }
        };

        let mbc = match select_mbc(raw_rom, &header) {
            Ok(mbc) => mbc,
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unsupported cartridge type: {e}"),
                )));
            }
        };

        let cartridge = Self { header, mbc };

        Ok(cartridge)
    }

    /// # Header checksum
    /// Checked by real hardware by the boot ROM
    pub fn check_header_checksum(&self, raw_rom: &[u8]) -> Result<()> {
        let header_sum = raw_rom[header::TITLE_START as usize..=header::GAME_VERSION]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));

        match header_sum == self.header.header_checksum {
            true => Ok(()),
            false => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Header checksum mismatch, {:#04X} != {:#04X}",
                    header_sum, self.header.header_checksum
                ),
            ))),
        }
    }

    /// # Global checksum
    /// Not actually checked by real hardware
    /// We'll use in Cartridge creation for now to verify correct file parsing and integrity
    pub fn check_global_checksum(&self, raw_rom: &[u8]) -> Result<()> {
        let cartridge_sum: u16 = raw_rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
            match i != header::GLOBAL_CHECKSUM_START as usize && i != header::GLOBAL_CHECKSUM_END as usize {
                true => acc.wrapping_add(b as u16),
                false => acc,
            }
        });

        match cartridge_sum == self.header.global_checksum {
            true => Ok(()),
            false => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Global checksum mismatch, {:#04X} != {:#04X}",
                    cartridge_sum, self.header.global_checksum
                ),
            ))),
        }
    }

    /// unmaps boot rom when boot reaches pc = 0x00FE, when load 1 in bank register (0xFF50)
    /// ```asm
    /// ld a, $01
    /// ld [0xFF50], a
    /// ```
    /// Next instruction will be the first `nop` in 0x0100, in the cartridge rom
    pub fn unmap_boot_rom(&mut self) {
        println!("Unmapping boot rom, switching to cartridge rom");
        // self.rom_bank00
        //     .copy_from_slice(&self.raw_rom[..ROM_BANK00_SIZE as usize]);
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
