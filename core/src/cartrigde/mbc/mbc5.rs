use crate::{
    EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    ROM_BANKNN_START,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, RomSize,
        features::{MbcFeatures, Rumble},
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
    features: MbcFeatures,
    rumble: Option<Rumble>,

    rom: Vec<u8>,
    rom_size: RomSize,
    rom_selected_bank: u16,

    ram: Vec<u8>,
    ram_enabled: bool,
    ram_size: RamSize,
    ram_selected_bank: u8,
}

impl MemoryBankController for Mbc5 {
    fn new(raw_rom: &[u8], header: &CartridgeHeader) -> CartridgeResult<Self> {
        let features = MbcFeatures::new(&header.cartridge_type);

        let rom = if raw_rom.len() == header.rom_size.get_size() as usize {
            raw_rom.to_vec()
        } else {
            return Err(CartridgeError::InvalidRomSize(
                Some(header.rom_size),
                "ROM size does not match the expected size for the cartridge",
            ));
        };

        let ram = vec![0; header.ram_size.get_size() as usize];

        Ok(Self {
            features,
            rumble: if header.cartridge_type.has_rumble() {
                Some(Rumble::new())
            } else {
                None
            },
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
            RAM_ENABLE_START..=RAM_ENABLE_END => {
                if self.features.has_ram {
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                }
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
                    if self.features.has_ram {
                        self.ram_selected_bank = value & 0x07;
                    }
                } else if self.features.has_ram {
                    self.ram_selected_bank = value & 0x0F;
                }
            }

            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() || !self.features.has_ram {
            return 0xFF;
        }

        let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
        if banks_count > 0 {
            let bank = (self.ram_selected_bank as usize) % banks_count;
            let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;

            return self.ram[offset];
        }

        0xFF
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() || !self.features.has_ram {
            return;
        }

        let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
        if banks_count > 0 {
            let bank = (self.ram_selected_bank as usize) % banks_count;
            let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;

            self.ram[offset] = value;
        }
    }
}
