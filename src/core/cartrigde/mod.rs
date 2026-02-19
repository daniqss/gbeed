mod header;
mod license;

use crate::{
    core::{Accessible, ROM_BANK00_SIZE},
    prelude::*,
};

use header::*;

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

impl Default for CartridgeType {
    fn default() -> Self { CartridgeType::RomOnly }
}

#[derive(Debug, Default)]
pub struct Cartridge {
    pub raw_rom: Vec<u8>,

    // header
    header: CartridgeHeader,

    // banks
    pub rom_bank00: Vec<u8>,
    rom_bank_nn: Vec<u8>,
    ram_bank: Vec<u8>,

    // checked in runtime to select currently used banks
    selected_rom_bank: u16,
    selected_ram_bank: u8,
    ram_enabled: bool,
}

impl Cartridge {
    pub fn new(raw_rom: Vec<u8>) -> Self {
        let header = CartridgeHeader::new(&raw_rom);

        let rom_bank00 = raw_rom[..ROM_BANK00_SIZE as usize].to_vec();
        let rom_bank_nn = raw_rom[ROM_BANK00_SIZE as usize..].to_vec();

        let ram_bank = vec![0; header.ram_size as usize];

        let cartridge = Self {
            header,
            raw_rom,
            rom_bank00,
            rom_bank_nn,
            ram_bank,
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            ram_enabled: false,
        };

        if let Err(e) = cartridge.check_header_checksum() {
            eprintln!(
                "Header checksum mismatch: the cartridge may be corrupted or the file may be malformed:\n{e}"
            );
        }

        cartridge
    }

    /// # Header checksum
    /// Checked by real hardware by the boot ROM
    pub fn check_header_checksum(&self) -> Result<()> {
        let header_sum = self.raw_rom[TITLE_START..=GAME_VERSION]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));

        match header_sum == self.header.header_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Header checksum mismatch, {:#04X} != {:#04X}",
                header_sum, self.header.header_checksum
            ))),
        }
    }

    /// # Global checksum
    /// Not actually checked by real hardware
    /// We'll use in Cartridge creation for now to verify correct file parsing and integrity
    pub fn check_global_checksum(&self) -> Result<()> {
        let cartridge_sum: u16 = self.raw_rom.iter().enumerate().fold(0u16, |acc, (i, &b)| {
            match i != GLOBAL_CHECKSUM_START && i != GLOBAL_CHECKSUM_END {
                true => acc.wrapping_add(b as u16),
                false => acc,
            }
        });

        match cartridge_sum == self.header.global_checksum {
            true => Ok(()),
            false => Err(Error::Generic(format!(
                "Global checksum mismatch, {:#04X} != {:#04X}",
                cartridge_sum, self.header.global_checksum,
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
        self.rom_bank00
            .copy_from_slice(&self.raw_rom[..ROM_BANK00_SIZE as usize]);
    }
}

impl Accessible<u16> for Cartridge {
    fn read(&self, address: u16) -> u8 { todo!() }
    fn write(&mut self, address: u16, value: u8) { todo!() }
}

impl std::fmt::Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { writeln!(f, "{}", self.header) }
}
