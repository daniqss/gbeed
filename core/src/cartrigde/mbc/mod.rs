mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

use crate::cartrigde::{
    CartridgeError, CartridgeResult, RomSize,
    features::CartridgeFeatures,
    header::{CARTRIDGE_TYPE, CartridgeHeader, DESTINATION_CODE},
};

use mbc0::Mbc0;
use mbc1::Mbc1;
use mbc2::Mbc2;
use mbc3::Mbc3;
use mbc5::Mbc5;

/// Indicates the available hardware in the cartridge
/// Is mostly used to indicates memory bank controllers
/// No licensed game uses RomRam and RomRamBattery
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CartridgeType {
    #[default]
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
    Mbc6,
    Mbc7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

impl CartridgeType {
    pub fn new(raw_rom: &[u8]) -> CartridgeType {
        use CartridgeType::*;
        match raw_rom[CARTRIDGE_TYPE] {
            0x00 => RomOnly,
            0x01 => Mbc1,
            0x02 => Mbc1Ram,
            0x03 => Mbc1RamBattery,
            0x05 => Mbc2,
            0x06 => Mbc2Battery,
            0x08 => RomRam,
            0x09 => RomRamBattery,
            0x0B => MMM01,
            0x0C => MMM01Ram,
            0x0D => MMM01RamBattery,
            0x0F => Mbc3TimerBattery,
            0x10 => Mbc3TimerRamBattery,
            0x11 => Mbc3,
            0x12 => Mbc3Ram,
            0x13 => Mbc3RamBattery,
            0x19 => Mbc5,
            0x1A => Mbc5Ram,
            0x1B => Mbc5RamBattery,
            0x1C => Mbc5Rumble,
            0x1D => Mbc5RumbleRam,
            0x1E => Mbc5RumbleRamBattery,
            0x20 => Mbc6,
            0x22 => Mbc7SensorRumbleRamBattery,
            0xFC => PocketCamera,
            0xFD => BandaiTama5,
            0xFE => HuC3,
            0xFF => HuC1RamBattery,

            _ => unreachable!("Unknown cartridge type: {:#X}", raw_rom[CARTRIDGE_TYPE]),
        }
    }

    pub fn has_ram(&self) -> bool {
        use CartridgeType::*;
        matches!(
            self,
            Mbc1Ram
                | Mbc1RamBattery
                | RomRam
                | RomRamBattery
                // they have build in RAM in MBC2
                | Mbc2
                | Mbc2Battery
                | Mbc3Ram
                | Mbc3RamBattery
                | Mbc3TimerRamBattery
                | Mbc5Ram
                | Mbc5RamBattery
                | Mbc5RumbleRam
                | Mbc5RumbleRamBattery
                | Mbc7SensorRumbleRamBattery
                | MMM01Ram
                | MMM01RamBattery
                | HuC1RamBattery
        )
    }

    pub fn has_battery(&self) -> bool {
        use CartridgeType::*;
        matches!(
            self,
            Mbc1RamBattery
                | RomRamBattery
                | Mbc3TimerBattery
                | Mbc3TimerRamBattery
                | Mbc3RamBattery
                | Mbc5RamBattery
                | Mbc5RumbleRamBattery
                | MMM01RamBattery
                | HuC1RamBattery
        )
    }

    pub fn has_timer(&self) -> bool {
        use CartridgeType::*;
        matches!(self, Mbc3TimerBattery | Mbc3TimerRamBattery)
    }

    pub fn has_rumble(&self) -> bool {
        use CartridgeType::*;
        matches!(
            self,
            Mbc5Rumble | Mbc5RumbleRam | Mbc5RumbleRamBattery | Mbc7SensorRumbleRamBattery
        )
    }

    pub fn has_sensor(&self) -> bool { matches!(self, CartridgeType::Mbc7SensorRumbleRamBattery) }
}

pub trait MemoryBankController {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self>
    where
        Self: Sized;

    fn read_rom(&self, address: u16) -> u8;
    fn write_rom(&mut self, address: u16, value: u8);
    fn read_ram(&self, address: u16) -> u8;
    fn write_ram(&mut self, address: u16, value: u8);

    fn get_ram(&self) -> Option<&[u8]>;
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]);
}

pub fn _check_multicart(raw_rom: &[u8], header: &CartridgeHeader) -> bool {
    let wisdom_tree = (header.title == "WISDOM TREE"
        && header.cartridge_type == CartridgeType::RomOnly
        && header.rom_size > RomSize::Rom32KB)
        || (raw_rom.get(CARTRIDGE_TYPE).copied() == Some(0xC0)
            && raw_rom.get(DESTINATION_CODE).copied() == Some(0xD1));

    let ems_multicart = header.cartridge_type == CartridgeType::Mbc5RamBattery
        && raw_rom.get(DESTINATION_CODE).copied() == Some(0xE1);

    let bung_multicart = raw_rom.get(CARTRIDGE_TYPE).copied() == Some(0xBE);

    wisdom_tree || ems_multicart || bung_multicart
}

pub fn select_mbc(
    raw_rom: &[u8],
    save: Option<Vec<u8>>,
    features: &CartridgeFeatures,
    header: &CartridgeHeader,
) -> CartridgeResult<Box<dyn MemoryBankController>> {
    use CartridgeType as CT;

    match header.cartridge_type {
        CT::RomOnly | CT::RomRam | CT::RomRamBattery => {
            Ok(Box::new(Mbc0::new(raw_rom, save, features, header)?))
        }
        CT::Mbc1 | CT::Mbc1Ram | CT::Mbc1RamBattery => {
            Ok(Box::new(Mbc1::new(raw_rom, save, features, header)?))
        }
        CT::Mbc2 | CT::Mbc2Battery => Ok(Box::new(Mbc2::new(raw_rom, save, features, header)?)),
        CT::Mbc3 | CT::Mbc3Ram | CT::Mbc3RamBattery | CT::Mbc3TimerBattery | CT::Mbc3TimerRamBattery => {
            Ok(Box::new(Mbc3::new(raw_rom, save, features, header)?))
        }
        CT::Mbc5
        | CT::Mbc5Ram
        | CT::Mbc5RamBattery
        | CT::Mbc5Rumble
        | CT::Mbc5RumbleRam
        | CT::Mbc5RumbleRamBattery => Ok(Box::new(Mbc5::new(raw_rom, save, features, header)?)),

        CT::MMM01 | CT::MMM01Ram | CT::MMM01RamBattery => {
            Err(CartridgeError::UnsupportedCartridgeType(header.cartridge_type))
        }

        CT::Mbc6 | CT::Mbc7SensorRumbleRamBattery => {
            Err(CartridgeError::UnsupportedCartridgeType(header.cartridge_type))
        }

        CT::PocketCamera | CT::BandaiTama5 | CT::HuC3 | CT::HuC1RamBattery => {
            Err(CartridgeError::UnsupportedCartridgeType(header.cartridge_type))
        }
    }
}
