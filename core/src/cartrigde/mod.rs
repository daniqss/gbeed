mod header;
mod mbc;

use crate::{
    prelude::*, EXTERNAL_RAM_END, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANK00_START, ROM_BANKNN_END,
    ROM_BANKNN_SIZE,
};

use header::CartridgeHeader;
pub use header::{RamSize, RomSize};
use mbc::{select_mbc, CartridgeType, MemoryBankController};

pub enum CartridgeError {
    InvalidRomSize(Option<RomSize>, &'static str),
    InvalidRamSize(Option<RamSize>, &'static str),
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
        }
    }
}

pub type CartridgeResult<T> = std::result::Result<T, CartridgeError>;

pub struct Cartridge {
    pub raw_rom: Vec<u8>,
    pub header: CartridgeHeader,
    pub mbc: Box<dyn MemoryBankController>,
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge").field("header", &self.header).finish()
    }
}

impl Default for Cartridge {
    fn default() -> Self { Cartridge::new(vec![0; (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize]).unwrap() }
}

impl Cartridge {
    pub fn new(raw_rom: Vec<u8>) -> Result<Self> {
        let header = match CartridgeHeader::new(&raw_rom) {
            Ok(header) => header,
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse cartridge header: {e}"),
                )));
            }
        };

        let mbc = match select_mbc(header.cartridge_type, header.rom, header.ram) {
            Ok(mbc) => mbc,
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unsupported cartridge type: {e}"),
                )));
            }
        };

        let cartridge = Self { raw_rom, header, mbc };

        #[cfg(not(test))]
        if let Err(e) = cartridge.check_header_checksum() {
            eprintln!(
                "Header checksum mismatch: the cartridge may be corrupted or the file may be malformed:\n{e}"
            );
        }

        Ok(cartridge)
    }

    /// # Header checksum
    /// Checked by real hardware by the boot ROM
    pub fn check_header_checksum(&self) -> Result<()> {
        let header_sum = self.raw_rom[header::TITLE_START as usize..=header::GAME_VERSION]
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
    pub fn check_global_checksum(&self) -> Result<()> {
        let cartridge_sum: u16 = self.raw_rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
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
