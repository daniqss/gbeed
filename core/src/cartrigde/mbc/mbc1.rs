use crate::{
    EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    ROM_BANKNN_START,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, RomSize,
        mbc::{CartridgeType, MbcFeatures},
    },
    mem_range,
};

use super::MemoryBankController;

mem_range!(RAM_ENABLE, 0x0000, 0x1FFF);
mem_range!(ROM_BANK_NUMBER, 0x2000, 0x3FFF);
mem_range!(RAM_BANK_NUMBER, 0x4000, 0x5FFF);
mem_range!(BANKING_MODE_SELECT, 0x6000, 0x7FFF);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum BankingMode {
    #[default]
    Simple,
    Advanced,
}

#[derive(Debug, Default)]
pub struct Mbc1 {
    features: MbcFeatures,

    mode: BankingMode,

    rom: Vec<u8>,
    rom_size: RomSize,
    primary_bank: u8,
    secondary_bank: u8,

    ram_enabled: bool,
    ram: Vec<u8>,
    ram_size: RamSize,
}

impl MemoryBankController for Mbc1 {
    fn new(
        raw_rom: &[u8],
        cartridge_type: CartridgeType,
        rom_size: RomSize,
        ram_size: RamSize,
    ) -> CartridgeResult<Self> {
        if rom_size > RomSize::Rom2MB || ram_size > RamSize::Ram32KB {
            return Err(CartridgeError::InvalidMBCRomRamCombination(
                cartridge_type,
                rom_size,
                ram_size,
                "MBC1 only supports up to 2MB ROM and 32KB RAM",
            ));
        }

        let rom = if raw_rom.len() == rom_size.get_size() as usize {
            raw_rom.to_vec()
        } else {
            return Err(CartridgeError::InvalidRomSize(
                Some(rom_size),
                "ROM size does not match the expected size for the cartridge",
            ));
        };

        let ram = vec![0; ram_size.get_size() as usize];

        Ok(Self {
            features: MbcFeatures::from(cartridge_type),
            mode: BankingMode::Simple,
            rom,
            rom_size,
            primary_bank: 0,
            secondary_bank: 0,
            ram_enabled: false,
            ram,
            ram_size,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            // first bank in simple mode, or bank selected by bank_reg_2 in advanced mode
            ROM_BANK00_START..=ROM_BANK00_END => {
                let bank = if self.mode == BankingMode::Advanced {
                    (self.secondary_bank << 5) as usize
                } else {
                    0
                };

                let bank = bank % self.rom_size.get_banks_count() as usize;
                let offset = (bank * ROM_BANKNN_SIZE as usize) + (address as usize);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }

            // bank selected by bank_reg_1 and bank_reg_2
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                // treat bank 0 as bank 1
                let primary_bank = if self.primary_bank == 0 {
                    1
                } else {
                    self.primary_bank
                };

                let bank = ((self.secondary_bank << 5) | primary_bank) as usize;

                let bank = bank % self.rom_size.get_banks_count() as usize;
                let offset = (bank * ROM_BANKNN_SIZE as usize) + (address - ROM_BANKNN_START) as usize;
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }

            _ => unreachable!(
                "MBC1: ROM read at address: {:#04X} should be handle by other components",
                address
            ),
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        match address {
            // any 0xA in lower bits enables RAM, any other value disables it
            RAM_ENABLE_START..=RAM_ENABLE_END if self.features.has_ram => {
                self.ram_enabled = (value & 0x0F) == 0x0A
            }

            ROM_BANK_NUMBER_START..=ROM_BANK_NUMBER_END => self.primary_bank = value & 0x1F,
            RAM_BANK_NUMBER_START..=RAM_BANK_NUMBER_END => self.secondary_bank = value & 0x03,

            BANKING_MODE_SELECT_START..=BANKING_MODE_SELECT_END => {
                self.mode = if value & 0x01 == 0 {
                    BankingMode::Simple
                } else {
                    BankingMode::Advanced
                };
            }
            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }

        let bank = match self.mode {
            BankingMode::Advanced => self.secondary_bank as usize,
            BankingMode::Simple => 0,
        };

        // if the cart is not large enough to use the 2 bit register (<= 8 KiB RAM) this mode select has no observable effect.
        let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
        if banks_count == 0 {
            return 0xFF;
        }

        let bank = bank % banks_count;
        let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }

        let bank = match self.mode {
            BankingMode::Advanced => self.secondary_bank as usize,
            BankingMode::Simple => 0,
        };

        let banks_count = self.ram_size.get_banks_count().unwrap_or(0) as usize;
        if banks_count == 0 {
            return;
        }

        let bank = bank % banks_count;
        let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
        if let Some(ram_cell) = self.ram.get_mut(offset) {
            *ram_cell = value;
        }
    }
}
