use crate::{
    EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    ROM_BANKNN_START,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, RomSize,
        features::{CartridgeFeatures, Rumble},
        header::CartridgeHeader,
    },
    prelude::*,
};

use super::MemoryBankController;

mem_range!(RAM_ENABLE, 0x0000, 0x1FFF);
mem_range!(ROM_BANK_NUMBER_LOW, 0x2000, 0x2FFF);
mem_range!(ROM_BANK_NUMBER_HIGH, 0x3000, 0x3FFF);
mem_range!(RAM_BANK_NUMBER, 0x4000, 0x5FFF);

#[derive(Debug, Default)]
pub struct Mbc5 {
    rumble: Option<Rumble>,

    rom: Vec<u8>,
    rom_size: RomSize,
    rom_selected_bank: u16,

    ram: Option<Vec<u8>>,
    ram_enabled: bool,
    ram_size: RamSize,
    ram_selected_bank: u8,
}

impl MemoryBankController for Mbc5 {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        let rom = if raw_rom.len() == header.rom_size.get_size() as usize {
            raw_rom.to_vec()
        } else {
            return Err(CartridgeError::InvalidRomSize(
                Some(header.rom_size),
                "ROM size does not match the expected size for the cartridge",
            ));
        };

        let ram = features.has_ram.then(|| {
            let ram_size = header.ram_size.get_size() as usize;
            save.filter(|s| features.has_battery && s.len() == ram_size)
                .unwrap_or_else(|| vec![0; ram_size])
        });

        let rumble = if header.cartridge_type.has_rumble() {
            Some(Rumble::new())
        } else {
            None
        };

        Ok(Self {
            rumble,
            rom,
            rom_size: header.rom_size,
            rom_selected_bank: 1,
            ram,
            ram_enabled: false,
            ram_size: header.ram_size,
            ram_selected_bank: 0,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANK00_END => self.rom[address as usize],
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                let bank = self.rom_selected_bank as usize % self.rom_size.get_banks_count() as usize;
                let offset = (bank * ROM_BANKNN_SIZE as usize) + (address - ROM_BANKNN_START) as usize;

                self.rom[offset]
            }

            _ => unreachable!(
                "MBC5: ROM read at address: {:#04X} should be handle by other components",
                address
            ),
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            RAM_ENABLE_START..=RAM_ENABLE_END if self.ram.is_some() => {
                self.ram_enabled = (value & 0x0F) == 0x0A
            }

            ROM_BANK_NUMBER_LOW_START..=ROM_BANK_NUMBER_LOW_END => {
                self.rom_selected_bank = (self.rom_selected_bank & 0xFF00) | (value as u16);
            }
            ROM_BANK_NUMBER_HIGH_START..=ROM_BANK_NUMBER_HIGH_END => {
                self.rom_selected_bank = (self.rom_selected_bank & 0x00FF) | (((value & 1) as u16) << 8);
            }
            RAM_BANK_NUMBER_START..=RAM_BANK_NUMBER_END => {
                if let Some(rumble) = &mut self.rumble {
                    rumble.enabled = (value & 0x08) != 0;
                    if self.ram.is_some() {
                        self.ram_selected_bank = value & 0x07;
                    }
                } else if self.ram.is_some() {
                    self.ram_selected_bank = value & 0x0F;
                }
            }

            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        match (&self.ram, self.ram_enabled, self.ram_size.get_banks_count()) {
            (Some(ram), true, Some(banks_count)) if !ram.is_empty() && banks_count > 0 => {
                let bank = (self.ram_selected_bank as usize) % banks_count as usize;
                let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
                ram[offset]
            }
            _ => 0xFF,
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        match (&mut self.ram, self.ram_enabled, self.ram_size.get_banks_count()) {
            (Some(ram), true, Some(banks_count)) if !ram.is_empty() && banks_count > 0 => {
                let bank = (self.ram_selected_bank as usize) % banks_count as usize;
                let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
                ram[offset] = value;
            }
            _ => {}
        }
    }

    fn get_ram(&self) -> Option<&[u8]> { self.ram.as_deref() }
}
