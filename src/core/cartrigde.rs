use super::license::get_license;
use crate::prelude::*;

const TITLE_START_ADDR: usize = 0x0134;
const TITLE_END_ADDR: usize = 0x0143;
const SGB_FLAG_ADDR: usize = 0x0146;
const CARTRIDGE_TYPE_ADDR: usize = 0x0147;
const ROM_SIZE_ADDR: usize = 0x0148;
const RAM_SIZE_ADDR: usize = 0x0149;
const DESTINATION_CODE_ADDR: usize = 0x014A;

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

pub struct Cartridge {
    raw: Vec<u8>,
    is_pre_sgb: bool,
    license: Option<String>,
    title: String,
    supports_sgb: bool,
    cartridge_type: CartridgeType,
    rom_size: u32,
    rom_banks: u16,
    ram_size: u32,
    destination: &'static str,
}

impl Cartridge {
    pub fn new(raw: Vec<u8>) -> Result<Self> {
        let (is_pre_sgb, license) = get_license(&raw);
        let title = &raw[TITLE_START_ADDR..TITLE_END_ADDR]
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        let supports_sgb = Self::get_supports_sgb(raw[SGB_FLAG_ADDR]);
        let cartridge_type = Self::get_cartridge_type(raw[CARTRIDGE_TYPE_ADDR]);
        let (rom_size, rom_banks) = Self::get_rom_size(raw[ROM_SIZE_ADDR]);
        let ram_size = Self::get_ram_size(raw[RAM_SIZE_ADDR]);
        let destination = Self::get_destination_code(raw[DESTINATION_CODE_ADDR]);

        Ok(Self {
            raw,
            is_pre_sgb,
            license,
            title: title.to_string(),
            supports_sgb,
            cartridge_type,
            rom_size,
            rom_banks,
            ram_size,
            destination,
        })
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

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &byte in &self.raw {
            let c = byte as char;
            match c.is_ascii_graphic() {
                true => write!(f, "{}", c)?,
                false => write!(f, ".")?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.title)?;
        write!(
            f,
            "{} -> {}\n",
            match self.is_pre_sgb {
                true => "Old license",
                false => "New license",
            },
            match &self.license {
                Some(l) => l,
                None => "None",
            }
        )?;
        write!(
            f,
            "{}upports Super Gameboy\n",
            match self.supports_sgb {
                true => "S",
                false => "Not s",
            }
        )?;
        write!(f, "Cartridge type -> {:#?}\n", self.cartridge_type)?;
        write!(
            f,
            "ROM Size -> {} KB ({} banks)\n",
            self.rom_size / 1024,
            self.rom_banks
        )?;
        write!(f, "RAM Size -> {} KB\n", self.ram_size / 1024)?;
        write!(f, "Destination code -> {}\n", self.destination)?;

        Ok(())
    }
}
