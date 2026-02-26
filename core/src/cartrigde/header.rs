use crate::{EXTERNAL_RAM_SIZE, ROM_BANKNN_SIZE};

use super::{license::get_license, mbc::Mbc};

pub const TITLE_START: usize = 0x0134;
pub const TITLE_END: usize = 0x0143;
pub const SGB_FLAG: usize = 0x0146;
pub const CARTRIDGE_TYPE: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const DESTINATION_CODE: usize = 0x014A;
pub const GAME_VERSION: usize = 0x014C;
pub const HEADER_CHECKSUM: usize = 0x14D;
pub const GLOBAL_CHECKSUM_START: usize = 0x14E;
pub const GLOBAL_CHECKSUM_END: usize = 0x14F;

#[derive(Debug, Default)]
pub struct CartridgeHeader {
    is_pre_sgb: bool,
    license: Option<String>,
    title: String,
    supports_sgb: bool,
    pub cartridge_type: Mbc,
    pub rom_size: u32,
    pub rom_banks_count: u16,
    pub external_ram_size: u32,
    pub external_ram_banks_count: Option<u16>,
    destination: &'static str,
    game_version: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(raw_rom: &Vec<u8>) -> Self {
        let (rom_size, rom_banks_count) = get_rom_size(raw_rom[ROM_SIZE_ADDRESS]);
        let (external_ram_size, external_ram_banks_count) = get_ram_size(raw_rom[RAM_SIZE_ADDRESS]);

        Self {
            is_pre_sgb: get_license(&raw_rom).0,
            license: get_license(&raw_rom).1,
            title: raw_rom[TITLE_START..TITLE_END]
                .iter()
                .map(|&c| c as char)
                .collect(),
            supports_sgb: get_supports_sgb(raw_rom[SGB_FLAG]),
            cartridge_type: get_cartridge_type(raw_rom[CARTRIDGE_TYPE]),
            rom_size,
            rom_banks_count,
            external_ram_size,
            external_ram_banks_count,
            destination: get_destination_code(raw_rom[DESTINATION_CODE]),
            game_version: raw_rom[GAME_VERSION],
            header_checksum: raw_rom[HEADER_CHECKSUM],
            global_checksum: ((raw_rom[GLOBAL_CHECKSUM_START] as u16) << 8)
                | (raw_rom[GLOBAL_CHECKSUM_END] as u16),
        }
    }
}

impl std::fmt::Display for CartridgeHeader {
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
            self.rom_size / 1024,
            self.rom_banks_count
        )?;
        writeln!(
            f,
            "External RAM Size -> {} {}",
            self.external_ram_size / 1024,
            match self.external_ram_banks_count {
                Some(count) => format!("({} banks)", count),
                None => "No RAM".to_string(),
            }
        )?;
        writeln!(f, "Destination code -> {}", self.destination)?;
        writeln!(f, "Game version -> {}", self.game_version)?;
        writeln!(f, "Header checksum -> {:#04X}", self.header_checksum)?;
        writeln!(f, "Global checksum -> {:#06X}", self.global_checksum)?;
        Ok(())
    }
}

/// indicates if the game supports Super Gameboy
fn get_supports_sgb(flag: u8) -> bool {
    match flag {
        0x03 => true,
        _ => false,
    }
}

fn get_cartridge_type(byte: u8) -> Mbc {
    match byte {
        0x00 => Mbc::RomOnly,
        0x01 => Mbc::Mbc1,
        0x02 => Mbc::Mbc1Ram,
        0x03 => Mbc::Mbc1RamBattery,
        0x05 => Mbc::Mbc2,
        0x06 => Mbc::Mbc2Battery,
        0x08 => Mbc::RomRam,
        0x09 => Mbc::RomRamBattery,
        0x0B => Mbc::MMM01,
        0x0C => Mbc::MMM01Ram,
        0x0D => Mbc::MMM01RamBattery,
        0x0F => Mbc::Mbc3TimerBattery,
        0x10 => Mbc::Mbc3TimerRamBattery,
        0x11 => Mbc::Mbc3,
        0x12 => Mbc::Mbc3Ram,
        0x13 => Mbc::Mbc3RamBattery,
        0x19 => Mbc::Mbc5,
        0x1A => Mbc::Mbc5Ram,
        0x1B => Mbc::Mbc5RamBattery,
        0x1C => Mbc::Mbc5Rumble,
        0x1D => Mbc::Mbc5RumbleRam,
        0x1E => Mbc::Mbc5RumbleRamBattery,
        0x1F => Mbc::PocketCamera,
        0xFD => Mbc::BandaiTama5,
        0xFE => Mbc::HuC3,
        0xFF => Mbc::HuC1RamBattery,
        _ => unreachable!("Unknown cartridge type: {byte:#X}"),
    }
}

/// # Rom Size
/// Other formats are listed in unofficial docs, but they're not found in real cartridges
fn get_rom_size(byte: u8) -> (u32, u16) {
    match byte {
        // 32 KiB rom size (2 banks) does not do banking
        0x00..=0x08 => (0x8000 << byte, 2 << byte),
        0x52 => (72 * ROM_BANKNN_SIZE as u32, 72),
        0x53 => (80 * ROM_BANKNN_SIZE as u32, 80),
        0x54 => (96 * ROM_BANKNN_SIZE as u32, 96),

        _ => unreachable!("Unknown ROM size: {byte:#X}"),
    }
}

/// # Ram Size
/// Defines how much RAM is provided by the cartridge
fn get_ram_size(byte: u8) -> (u32, Option<u16>) {
    match byte {
        0x00 => (0, None),
        0x01 => unreachable!("Unused RAM size: {byte:#X}"),
        0x02 => (EXTERNAL_RAM_SIZE as u32, Some(1)),
        0x03 => (4 * EXTERNAL_RAM_SIZE as u32, Some(4)),
        0x04 => (16 * EXTERNAL_RAM_SIZE as u32, Some(16)),
        0x05 => (8 * EXTERNAL_RAM_SIZE as u32, Some(8)),
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
