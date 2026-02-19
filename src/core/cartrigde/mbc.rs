use crate::Cartridge;

/// Indicates the available hardware in the cartridge
/// Is mostly used to indicates memory bank controllers
/// No licensed game uses RomRam and RomRamBattery
/// Mbc3Ram, Mbc3TimerBattery, Mbc3TimerRamBattery with 64kb of RAM is Pokemon Crystal Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mbc {
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

impl Default for Mbc {
    fn default() -> Self { Mbc::RomOnly }
}

pub fn enable_ram(cartridge: &mut Cartridge, address: u16, value: u8) {
    cartridge.ram_enabled = match cartridge.header.cartridge_type {
        Mbc::RomOnly | Mbc::RomRam | Mbc::RomRamBattery => (value & 0x0F) == 0x0A,

        Mbc::Mbc1 | Mbc::Mbc1Ram | Mbc::Mbc1RamBattery => (value & 0x0F) == 0x0A,

        Mbc::Mbc2 | Mbc::Mbc2Battery if address & 0x0100 == 0 => (value & 0x0F) == 0x0A,

        Mbc::Mbc3 | Mbc::Mbc3Ram | Mbc::Mbc3RamBattery | Mbc::Mbc3TimerBattery | Mbc::Mbc3TimerRamBattery => {
            (value & 0x0F) == 0x0A
        }
        Mbc::Mbc5
        | Mbc::Mbc5Ram
        | Mbc::Mbc5RamBattery
        | Mbc::Mbc5Rumble
        | Mbc::Mbc5RumbleRam
        | Mbc::Mbc5RumbleRamBattery => (value & 0x0F) == 0x0A,

        // TODO: handle every cartridge type
        _ => cartridge.ram_enabled,
    };
}

pub fn select_rom_bank(cartridge: &mut Cartridge, address: u16, value: u8) {
    match cartridge.header.cartridge_type {
        Mbc::Mbc1 | Mbc::Mbc1Ram | Mbc::Mbc1RamBattery => {
            let mut val = value & 0x1F;
            if val == 0 {
                val = 1;
            }
            cartridge.selected_rom_bank = (cartridge.selected_rom_bank & 0x60) | (val as u16);
        }
        Mbc::Mbc2 | Mbc::Mbc2Battery => {
            if address & 0x0100 != 0 {
                let mut val = value & 0x0F;
                if val == 0 {
                    val = 1;
                }
                cartridge.selected_rom_bank = val as u16;
            }
        }
        Mbc::Mbc3 | Mbc::Mbc3Ram | Mbc::Mbc3RamBattery | Mbc::Mbc3TimerBattery | Mbc::Mbc3TimerRamBattery => {
            let mut val = value & 0x7F;
            if val == 0 {
                val = 1;
            }
            cartridge.selected_rom_bank = val as u16;
        }
        Mbc::Mbc5
        | Mbc::Mbc5Ram
        | Mbc::Mbc5RamBattery
        | Mbc::Mbc5Rumble
        | Mbc::Mbc5RumbleRam
        | Mbc::Mbc5RumbleRamBattery => {
            if address < 0x3000 {
                cartridge.selected_rom_bank = (cartridge.selected_rom_bank & 0x100) | (value as u16);
            } else {
                cartridge.selected_rom_bank =
                    (cartridge.selected_rom_bank & 0xFF) | (((value & 1) as u16) << 8);
            }
        }
        _ => {}
    }
}

pub fn select_ram_bank(cartridge: &mut Cartridge, value: u8) {
    match cartridge.header.cartridge_type {
        Mbc::Mbc1 | Mbc::Mbc1Ram | Mbc::Mbc1RamBattery => {
            let val = value & 0x03;
            cartridge.selected_ram_bank = val as u16;
            cartridge.selected_rom_bank = (cartridge.selected_rom_bank & 0x1F) | ((val as u16) << 5);
        }
        Mbc::Mbc3 | Mbc::Mbc3Ram | Mbc::Mbc3RamBattery | Mbc::Mbc3TimerBattery | Mbc::Mbc3TimerRamBattery => {
            if value <= 0x03 {
                cartridge.selected_ram_bank = value as u16;
            } else if value >= 0x08 && value <= 0x0C {
                cartridge.selected_ram_bank = value as u16;
            }
        }
        Mbc::Mbc5
        | Mbc::Mbc5Ram
        | Mbc::Mbc5RamBattery
        | Mbc::Mbc5Rumble
        | Mbc::Mbc5RumbleRam
        | Mbc::Mbc5RumbleRamBattery => {
            cartridge.selected_ram_bank = (value & 0x0F) as u16;
        }
        _ => {}
    }
}

pub fn select_banking_mode(cartridge: &mut Cartridge, value: u8) {
    match cartridge.header.cartridge_type {
        Mbc::Mbc1 | Mbc::Mbc1Ram | Mbc::Mbc1RamBattery => {
            cartridge.banking_mode = (value & 0x01) != 0;
        }
        _ => {}
    }
}
