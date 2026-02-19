use super::{CartridgeType, license::get_license};

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

#[derive(Debug, Default)]
pub struct CartridgeHeader {
    is_pre_sgb: bool,
    license: Option<String>,
    title: String,
    supports_sgb: bool,
    cartridge_type: CartridgeType,
    pub rom_size: (u32, u16),
    pub ram_size: u32,
    destination: &'static str,
    game_version: u8,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(raw_rom: &Vec<u8>) -> Self {
        Self {
            is_pre_sgb: get_license(&raw_rom).0,
            license: get_license(&raw_rom).1,
            title: raw_rom[TITLE_START..TITLE_END]
                .iter()
                .map(|&c| c as char)
                .collect(),
            supports_sgb: get_supports_sgb(raw_rom[SGB_FLAG]),
            cartridge_type: get_cartridge_type(raw_rom[CARTRIDGE_TYPE]),
            rom_size: get_rom_size(raw_rom[ROM_SIZE]),
            ram_size: get_ram_size(raw_rom[RAM_SIZE]),
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
        // 32 KiB rom size (2 banks) does not do banking
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
