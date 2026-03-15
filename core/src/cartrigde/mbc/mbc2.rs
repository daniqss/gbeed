use crate::{
    ROM_BANK00_END, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE, ROM_BANKNN_START,
    cartrigde::{CartridgeResult, header::CartridgeHeader, mbc::MbcFeatures},
};

use super::MemoryBankController;

#[derive(Debug)]
pub struct Mbc2 {
    _features: MbcFeatures,
    rom: Vec<u8>,
    rom_size_banks: usize,
    rom_bank: u8,

    ram: [u8; 512],
    ram_enabled: bool,
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self {
            _features: MbcFeatures::default(),
            rom: Vec::new(),
            rom_size_banks: 0,
            rom_bank: 1,
            ram: [0; 512],
            ram_enabled: false,
        }
    }
}

impl MemoryBankController for Mbc2 {
    fn new(raw_rom: &[u8], header: &CartridgeHeader) -> CartridgeResult<Self> {
        let rom = raw_rom.to_vec();
        // Use the actual number of banks available in the provided buffer
        let rom_size_banks = rom.len() / ROM_BANKNN_SIZE as usize;

        Ok(Self {
            _features: MbcFeatures::new(&header.cartridge_type),
            rom,
            rom_size_banks,
            rom_bank: 1,
            ram: [0; 512],
            ram_enabled: false,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANK00_END => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                // 16 banks max, changing bank 0 to 1 since bank 0 is fixed to the first 16KB of the ROM
                let bank = (self.rom_bank as usize % 16).max(1);

                let bank = bank % self.rom_size_banks;
                let offset = (bank * ROM_BANKNN_SIZE as usize) + (address - ROM_BANKNN_START) as usize;
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }

            _ => unreachable!(
                "MBC2: ROM read at address: {:#04X} should be handle by other components",
                address
            ),
        }
    }

    fn write_rom(&mut self, address: u16, value: u8) {
        if let ROM_BANK00_START..=ROM_BANK00_END = address {
            if address & 0x0100 == 0 {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            } else {
                // only lower 4 bits for bank selection
                self.rom_bank = value & 0x0F;
            }
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        let offset = (address & 0x01FF) as usize;
        // clean upper 4 bits
        self.ram[offset] | 0xF0
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        let offset = (address & 0x01FF) as usize;
        self.ram[offset] = value & 0x0F;
    }
}
