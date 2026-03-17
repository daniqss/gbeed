use crate::{
    BOOT_ROM_END, BOOT_ROM_START, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_END, ROM_BANK00_START,
    ROM_BANKNN_END, ROM_BANKNN_SIZE, ROM_BANKNN_START,
    cartrigde::{
        CARTRIDGE_LOGO_START, CartridgeError, CartridgeResult, NINTENDO_LOGO, RamSize, RomSize,
        features::CartridgeFeatures, header::CartridgeHeader,
    },
    mem_range,
};

use super::MemoryBankController;

mem_range!(MBC1_RAM_ENABLE, 0x0000, 0x1FFF);
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
    mode: BankingMode,
    is_multicart: bool,

    rom: Vec<u8>,
    rom_size: RomSize,
    primary_bank: u8,
    secondary_bank: u8,

    ram_enabled: bool,
    ram: Option<Vec<u8>>,
    ram_size: RamSize,
}

/// Checks for Nintendo logo in the various ROMs of the multicart
/// Tested against [this test](https://github.com/Gekkio/mooneye-test-suite/blob/main/emulator-only/mbc1/multicart_rom_8Mb.s)
fn check_mbc1m_multicart(raw_rom: &[u8], header: &CartridgeHeader) -> bool {
    if header.rom_size != RomSize::Rom1MB {
        return false;
    }

    let mut logo_matches = 0;

    let logo_offset = ROM_BANKNN_SIZE as usize * 0x10;
    for i in 0..4 {
        let offset = i * logo_offset + (CARTRIDGE_LOGO_START as usize);
        if let Some(slice) = raw_rom.get(offset..offset + NINTENDO_LOGO.len())
            && slice == NINTENDO_LOGO
        {
            logo_matches += 1;
        }
    }
    // if we find at least two matches we can be reasonably sure this is a MBC1M multicart
    logo_matches >= 2
}

impl MemoryBankController for Mbc1 {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        if header.rom_size > RomSize::Rom2MB || header.ram_size > RamSize::Ram32KB {
            return Err(CartridgeError::InvalidMBCRomRamCombination(
                header.cartridge_type,
                header.rom_size,
                header.ram_size,
                "MBC1 only supports up to 2MB ROM and 32KB RAM",
            ));
        }

        let is_multicart = check_mbc1m_multicart(raw_rom, header);

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

        Ok(Self {
            mode: BankingMode::Simple,
            is_multicart,
            rom,
            rom_size: header.rom_size,
            primary_bank: 0,
            secondary_bank: 0,
            ram_enabled: false,
            ram,
            ram_size: header.ram_size,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        let banks_count = self.rom_size.get_banks_count() as usize;

        match address {
            // first bank in simple mode, or bank selected by secondary_bank in advanced mode
            ROM_BANK00_START..=ROM_BANK00_END => {
                let bank = if self.mode == BankingMode::Advanced {
                    if self.is_multicart {
                        (self.secondary_bank << 4) as usize
                    } else {
                        (self.secondary_bank << 5) as usize
                    }
                } else {
                    0
                };

                let bank = bank % banks_count;
                let offset = (bank * ROM_BANKNN_SIZE as usize) + (address as usize);
                self.rom[offset]
            }

            // bank selected by primary_bank and secondary_bank
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                // bank 0 is treated as bank 1
                let primary_bank = (self.primary_bank as usize).max(1);

                let bank = if self.is_multicart {
                    (self.secondary_bank << 4) as usize | (primary_bank & 0x0F)
                } else {
                    (self.secondary_bank << 5) as usize | (primary_bank & 0x1F)
                };
                let bank = bank % banks_count;

                let offset = bank * ROM_BANKNN_SIZE as usize + (address - ROM_BANKNN_START) as usize;
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
            MBC1_RAM_ENABLE_START..=MBC1_RAM_ENABLE_END => {
                if self.ram.is_some() {
                    self.ram_enabled = (value & 0x0F) == 0x0A
                }
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
        match (&self.ram, self.ram_enabled, self.ram_size.get_banks_count()) {
            (Some(ram), true, Some(banks_count)) if banks_count > 0 => {
                let bank = match self.mode {
                    BankingMode::Advanced => self.secondary_bank as usize % banks_count as usize,
                    BankingMode::Simple => 0,
                };

                let offset = (bank * EXTERNAL_RAM_SIZE as usize) + (address - EXTERNAL_RAM_START) as usize;
                ram[offset]
            }
            _ => 0xFF,
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_none() {
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

        if let Some(ram) = &mut self.ram {
            ram[offset] = value;
        }
    }

    fn get_ram(&self) -> Option<&[u8]> { self.ram.as_deref() }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
