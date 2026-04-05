use crate::{
    BOOT_ROM_END, BOOT_ROM_START, ROM_BANK00_END, ROM_BANK00_START, ROM_BANKNN_END, ROM_BANKNN_SIZE,
    ROM_BANKNN_START,
    cartrigde::{
        CartridgeError, CartridgeResult, RomSize, features::CartridgeFeatures, header::CartridgeHeader,
    },
};

use super::MemoryBankController;

const MBC2_RAM_SIZE: usize = 512;

#[derive(Debug)]
pub struct Mbc2 {
    rom: Vec<u8>,
    rom_size: RomSize,
    rom_selected_bank: u8,

    ram: Box<[u8; MBC2_RAM_SIZE]>,
    ram_enabled: bool,
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self {
            rom: Vec::new(),
            rom_size: RomSize::Rom512KB,
            rom_selected_bank: 1,
            ram: Box::new([0; MBC2_RAM_SIZE]),
            ram_enabled: false,
        }
    }
}

impl MemoryBankController for Mbc2 {
    fn new(
        raw_rom: &[u8],
        _: Option<Vec<u8>>,
        _: &CartridgeFeatures,
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

        Ok(Self {
            rom,
            rom_size: header.rom_size,

            rom_selected_bank: 1,
            ram: Box::new([0; MBC2_RAM_SIZE]),
            ram_enabled: false,
        })
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            ROM_BANK00_START..=ROM_BANK00_END => self.rom[address as usize],
            ROM_BANKNN_START..=ROM_BANKNN_END => {
                // 16 banks max, changing bank 0 to 1 since bank 0 is fixed to the first 16KB of the ROM
                let bank = self.rom_selected_bank.max(1);

                let bank = bank % self.rom_size.get_banks_count() as u8;
                let offset =
                    (bank as usize * ROM_BANKNN_SIZE as usize) + (address - ROM_BANKNN_START) as usize;
                self.rom[offset]
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
                self.rom_selected_bank = value & 0x0F;
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

    fn get_ram(&self) -> Option<&[u8]> { Some(self.ram.as_slice()) }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
