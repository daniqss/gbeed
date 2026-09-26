#[cfg(feature = "alloc")]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
mod license;
mod ram;
mod rom;

use crate::{cartrigde::CartridgeResult, mem_range};
pub use ram::RamSize;
pub use rom::RomSize;
use {super::mbc::CartridgeType, license::get_license};

mem_range!(TITLE, 0x0134, 0x0143);
pub const GBC_FLAG: usize = 0x0143;
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
        match raw_rom[GBC_FLAG] {
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

/// Game title, taken verbatim from the cartridge header (0x0134..=0x0143, 16 bytes at most)
#[derive(Debug, Default, Clone, Copy)]
pub struct Title {
    bytes: [u8; TITLE_SIZE as usize],
    len: u8,
}

impl Title {
    fn new(raw_rom: &[u8]) -> Self {
        let raw_title = &raw_rom[TITLE_START as usize..TITLE_END as usize];
        let len = raw_title.iter().take_while(|&&c| c != 0).count();

        let mut bytes = [0u8; TITLE_SIZE as usize];
        bytes[..len].copy_from_slice(&raw_title[..len]);

        Self {
            bytes,
            len: len as u8,
        }
    }

    pub fn as_str(&self) -> &str { core::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("") }
}

impl core::ops::Deref for Title {
    type Target = str;
    fn deref(&self) -> &str { self.as_str() }
}

impl core::fmt::Display for Title {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{}", self.as_str()) }
}

impl PartialEq<&str> for Title {
    fn eq(&self, other: &&str) -> bool { self.as_str() == *other }
}

// `is_pre_sgb`, `license`, `supports_cgb`, `supports_sgb` and `game_version` are only read by
// `to_string_array`, which needs `alloc`
#[cfg_attr(not(feature = "alloc"), allow(dead_code))]
#[derive(Debug, Default)]
pub struct CartridgeHeader {
    is_pre_sgb: bool,
    license: Option<&'static str>,
    pub title: Title,
    supports_cgb: GBCSupport,
    supports_sgb: bool,
    pub(crate) cartridge_type: CartridgeType,
    pub(crate) rom_size: RomSize,
    pub(crate) ram_size: RamSize,
    pub destination: Destination,
    game_version: u8,
    pub(crate) header_checksum: u8,
    pub(crate) global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(raw_rom: &[u8]) -> CartridgeResult<Self> {
        let rom_size = RomSize::new(raw_rom[ROM_SIZE_ADDRESS])?;
        let ram_size = RamSize::new(raw_rom[RAM_SIZE_ADDRESS])?;

        Ok(Self {
            is_pre_sgb: get_license(raw_rom).0,
            license: get_license(raw_rom).1,
            title: Title::new(raw_rom),
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

    #[cfg(feature = "alloc")]
    pub fn to_string_array(&self) -> Vec<String> {
        vec![
            format!(
                "{} -> {}",
                match self.is_pre_sgb {
                    true => "Old license",
                    false => "New license",
                },
                self.license.unwrap_or("None")
            ),
            format!("Supports CGB -> {:?}", self.supports_cgb),
            format!(
                "{}upports Super Gameboy",
                match self.supports_sgb {
                    true => "S",
                    false => "Not s",
                }
            ),
            format!("Cartridge type -> {:#?}", self.cartridge_type),
            format!(
                "ROM Size -> {} KB ({} banks)",
                self.rom_size.get_size() / 1024,
                self.rom_size.get_banks_count()
            ),
            format!(
                "External RAM Size -> {} KB {}",
                self.ram_size.get_size() / 1024,
                match self.ram_size.get_banks_count() {
                    Some(count) => format!("({} banks)", count),
                    None => "No RAM".to_string(),
                }
            ),
            format!("Destination code -> {:?}", self.destination),
            format!("Game version -> {}", self.game_version),
            format!("Header checksum -> {:#04X}", self.header_checksum),
            format!("Global checksum -> {:#06X}", self.global_checksum),
        ]
    }
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_string_array().join("\n"))
    }
}

/// indicates if the game supports Super Gameboy
fn get_supports_sgb(raw_rom: &[u8]) -> bool { matches!(raw_rom[SGB_FLAG], 0x03) }
