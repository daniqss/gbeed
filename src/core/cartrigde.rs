use super::license::get_license;
use crate::prelude::*;

pub const TITLE_START: usize = 0x0134;
pub const TITLE_END: usize = 0x0143;
pub const SGB_FLAG: usize = 0x0146;
pub const CARTRIDGE_TYPE: usize = 0x0147;
pub const ROM_SIZE: usize = 0x0148;
pub const RAM_SIZE: usize = 0x0149;
pub const DESTINATION_CODE: usize = 0x014A;
pub const GAME_VERSION: usize = 0x014C;
pub const HEADER_CHECKSUM: usize = 0x14D;
pub const GLOBAL_CHECKSUM_START: usize = 0x14E;
pub const GLOBAL_CHECKSUM_END: usize = 0x14F;

/// Indicates the available hardware in the cartridge
/// Is mostly used to indicates memory bank controllers
/// No licensed game uses RomRam and RomRamBattery
/// Mbc3Ram, Mbc3TimerBattery, Mbc3TimerRamBattery with 64kb of RAM is Pokemon Crystal Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    MMM01,
    MMM01Ram,
    MMM01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

#[derive(Debug)]
pub struct Cartridge {
    pub rom: Vec<u8>,

    is_pre_sgb: bool,
    license: Option<String>,
    title: String,
    supports_sgb: bool,
    cartridge_type: CartridgeType,
    rom_size: (u32, u16),
    ram_size: u32,
    destination: &'static str,
    game_version: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl Cartridge {
    pub fn new(game_rom: Vec<u8>) -> Result<Self> {
        let cartridge = Self {
            is_pre_sgb: get_license(&game_rom).0,
            license: get_license(&game_rom).1,
            title: game_rom[TITLE_START..TITLE_END]
                .iter()
                .map(|&c| c as char)
                .collect(),
            supports_sgb: Self::get_supports_sgb(game_rom[SGB_FLAG]),
            cartridge_type: Self::get_cartridge_type(game_rom[CARTRIDGE_TYPE]),
            rom_size: Self::get_rom_size(game_rom[ROM_SIZE]),
            ram_size: Self::get_ram_size(game_rom[RAM_SIZE]),
            destination: Self::get_destination_code(game_rom[DESTINATION_CODE]),
            game_version: game_rom[GAME_VERSION],
            header_checksum: game_rom[HEADER_CHECKSUM],
            global_checksum: ((game_rom[GLOBAL_CHECKSUM_START] as u16) << 8)
                | (game_rom[GLOBAL_CHECKSUM_END] as u16),

            rom: game_rom,
        };

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
        let header_sum = self.rom[TITLE_START..=GAME_VERSION]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));

        match header_sum == self.header_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Header checksum mismatch, {:#04X} != {:#04X}",
                header_sum, self.header_checksum
            ))),
        }
    }

    /// # Global checksum
    /// Not actually checked by real hardware
    /// We'll use in Cartridge creation for now to verify correct file parsing and integrity
    pub fn check_global_checksum(&self) -> Result<()> {
        let cartridge_sum: u16 = self.rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
            match i != GLOBAL_CHECKSUM_START && i != GLOBAL_CHECKSUM_END {
                true => acc.wrapping_add(b as u16),
                false => acc,
            }
        });

        match cartridge_sum == self.global_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Global checksum mismatch, {:#04X} != {:#04X}",
                cartridge_sum, self.global_checksum,
            ))),
        }
    }

    /// indicates if the game supports Super Gameboy
    fn get_supports_sgb(flag: u8) -> bool {
        match flag {
            0x03 => true,
            _ => false,
        }
    }

    fn get_cartridge_type(byte: u8) -> CartridgeType {
        match byte {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x08 => CartridgeType::RomRam,
            0x09 => CartridgeType::RomRamBattery,
            0x0B => CartridgeType::MMM01,
            0x0C => CartridgeType::MMM01Ram,
            0x0D => CartridgeType::MMM01RamBattery,
            0x0F => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1A => CartridgeType::Mbc5Ram,
            0x1B => CartridgeType::Mbc5RamBattery,
            0x1C => CartridgeType::Mbc5Rumble,
            0x1D => CartridgeType::Mbc5RumbleRam,
            0x1E => CartridgeType::Mbc5RumbleRamBattery,
            0x1F => CartridgeType::PocketCamera,
            0xFD => CartridgeType::BandaiTama5,
            0xFE => CartridgeType::HuC3,
            0xFF => CartridgeType::HuC1RamBattery,
            _ => unreachable!("Unknown cartridge type: {byte:#X}"),
        }
    }

    /// # Rom Size
    /// Other formats are listed in unofficial docs, but they're not found in real cartridges
    fn get_rom_size(byte: u8) -> (u32, u16) {
        match byte {
            // 32 KiB rom size (2 banks) does not do bancking
            0x00..=0x07 => (0x8000 << byte, 2 << byte),
            _ => unreachable!("Unknown ROM size: {byte:#X}"),
        }
    }

    /// # Ram Size
    /// Defines how much RAM is provided by the cartridge
    fn get_ram_size(byte: u8) -> u32 {
        match byte {
            0x00 => 0,
            0x01 => unreachable!("Unused RAM size: {byte:#X}"),
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => unreachable!("Unknown RAM size: {byte:#X}"),
        }
    }

    /// # Destination Code
    /// Whether the game is made for japanese or overseas markets
    fn get_destination_code(byte: u8) -> &'static str {
        match byte {
            0x00 => "Japanese",
            0x01 => "Overseas",
            _ => unreachable!("Unknown destination code: {byte:#X}"),
        }
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.title.replace('\0', ""))?;
        writeln!(
            f,
            "{} -> {}",
            match self.is_pre_sgb {
                true => "Old license",
                false => "New license",
            },
            match &self.license {
                Some(l) => l,
                None => "None",
            }
        )?;
        writeln!(
            f,
            "{}upports Super Gameboy",
            match self.supports_sgb {
                true => "S",
                false => "Not s",
            }
        )?;
        writeln!(f, "Cartridge type -> {:#?}", self.cartridge_type)?;
        writeln!(
            f,
            "ROM Size -> {} KB ({} banks)",
            self.rom_size.0 / 1024,
            self.rom_size.1
        )?;
        writeln!(f, "RAM Size -> {} KB", self.ram_size / 1024)?;
        writeln!(f, "Destination code -> {}", self.destination)?;
        writeln!(f, "Game version -> {}", self.game_version)?;
        writeln!(f, "Header checksum -> {:#04X}", self.header_checksum)?;
        writeln!(f, "Global checksum -> {:#06X}", self.global_checksum)?;
        Ok(())
    }
}
