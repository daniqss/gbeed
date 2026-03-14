mod license;
mod ram;
mod rom;

use crate::{cartrigde::CartridgeResult, mem_range};
pub use ram::RamSize;
pub use rom::RomSize;
use {super::mbc::CartridgeType, license::get_license};

mem_range!(TITLE, 0x0134, 0x0143);
pub const SGB_FLAG: usize = 0x0146;
pub const CARTRIDGE_TYPE: usize = 0x0147;
pub const ROM_SIZE_ADDRESS: usize = 0x0148;
pub const RAM_SIZE_ADDRESS: usize = 0x0149;
pub const DESTINATION_CODE: usize = 0x014A;
pub const GAME_VERSION: usize = 0x014C;
pub const HEADER_CHECKSUM: usize = 0x14D;
mem_range!(GLOBAL_CHECKSUM, 0x14E, 0x14F);

#[derive(Debug, Default)]
enum GBCSupport {
    #[default]
    None,
    Enhancements = 0x80,
    Only = 0xC0,
}

impl GBCSupport {
    fn new(raw_rom: &[u8]) -> GBCSupport {
        match raw_rom[CARTRIDGE_TYPE] {
            0x80 => GBCSupport::Enhancements,
            0xC0 => GBCSupport::Only,
            _ => GBCSupport::None,
        }
    }
}

/// # Destination Code
/// Whether the game is made for japanese or overseas markets
#[derive(Debug, Default, Clone, Copy)]
pub enum Destination {
    #[default]
    Japan = 0x00,
    Overseas = 0x01,
    Undefined,
}

impl Destination {
    fn new(byte: u8) -> Self {
        match byte {
            0x00 => Destination::Japan,
            0x01 => Destination::Overseas,
            _ => Destination::Undefined,
        }
    }
}

#[derive(Debug, Default)]
pub struct CartridgeHeader {
    is_pre_sgb: bool,
    license: Option<String>,
    pub title: String,
    supports_cgb: GBCSupport,
    supports_sgb: bool,
    pub cartridge_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub destination: Destination,
    game_version: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(raw_rom: &[u8]) -> CartridgeResult<Self> {
        let rom_size = RomSize::new(raw_rom[ROM_SIZE_ADDRESS])?;
        let ram_size = RamSize::new(raw_rom[RAM_SIZE_ADDRESS])?;

        Ok(Self {
            is_pre_sgb: get_license(raw_rom).0,
            license: get_license(raw_rom).1,
            title: raw_rom[TITLE_START as usize..TITLE_END as usize]
                .iter()
                .map(|&c| c as char)
                .collect(),
            supports_cgb: GBCSupport::new(raw_rom),
            supports_sgb: get_supports_sgb(raw_rom),
            cartridge_type: CartridgeType::new(raw_rom),
            rom_size,
            ram_size,
            destination: Destination::new(raw_rom[DESTINATION_CODE]),
            game_version: raw_rom[GAME_VERSION],
            header_checksum: raw_rom[HEADER_CHECKSUM],
            global_checksum: ((raw_rom[GLOBAL_CHECKSUM_START as usize] as u16) << 8)
                | (raw_rom[GLOBAL_CHECKSUM_END as usize] as u16),
        })
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
        writeln!(f, "Supports CGB -> {:?}", self.supports_cgb)?;
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
            self.rom_size.get_size() / 1024,
            self.rom_size.get_banks_count()
        )?;
        writeln!(
            f,
            "External RAM Size -> {} KB {}",
            self.ram_size.get_size() / 1024,
            match self.ram_size.get_banks_count() {
                Some(count) => format!("({} banks)", count),
                None => "No RAM".to_string(),
            }
        )?;
        writeln!(f, "Destination code -> {:?}", self.destination)?;
        writeln!(f, "Game version -> {}", self.game_version)?;
        writeln!(f, "Header checksum -> {:#04X}", self.header_checksum)?;
        writeln!(f, "Global checksum -> {:#06X}", self.global_checksum)?;
        Ok(())
    }
}

/// indicates if the game supports Super Gameboy
fn get_supports_sgb(raw_rom: &[u8]) -> bool { matches!(raw_rom[SGB_FLAG], 0x03) }
