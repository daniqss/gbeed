use crate::{
    BOOT_ROM_END, BOOT_ROM_START, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANKNN_SIZE,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, features::CartridgeFeatures, header::CartridgeHeader,
    },
    utils::Buffer,
};

use super::MemoryBankController;

const MBC0_ROM_SIZE: usize = (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize;
const MBC0_RAM_SIZE: usize = EXTERNAL_RAM_SIZE as usize;

/// Memory Bank Controller for cartridges without any MBC (ROM only mostly).
/// They can have a RAM chip using a discrete logic decode but without a full MCB.
#[derive(Debug)]
pub struct Mbc0 {
    rom: Buffer<MBC0_ROM_SIZE>,
    ram: Option<Buffer<MBC0_RAM_SIZE>>,
}

impl MemoryBankController for Mbc0 {
    fn new(
        raw_rom: &[u8],
        save: Option<&[u8]>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        if raw_rom.len() < MBC0_ROM_SIZE {
            return Err(CartridgeError::InvalidRomSize(
                Some(header.rom_size),
                "ROM is smaller than the 32KB of a cartridge without a memory bank controller",
            ));
        }

        let rom = Buffer::from_slice(&raw_rom[..MBC0_ROM_SIZE]);

        let ram: Option<Buffer<MBC0_RAM_SIZE>> = match (features.has_ram, header.ram_size) {
            (true, RamSize::Ram8KB) => Some(match save {
                Some(save_data) if save_data.len() == MBC0_RAM_SIZE => Buffer::from_slice(save_data),
                _ => Buffer::zeroed(MBC0_RAM_SIZE),
            }),
            (false, RamSize::None) => None,
            (_, ram) => {
                return Err(CartridgeError::InvalidRamSize(
                    Some(ram),
                    "Only 8KB RAM size is supported for MBC0",
                ));
            }
        };

        Ok(Self { rom, ram })
    }

    fn read_rom(&self, address: u16) -> u8 { self.rom[address as usize] }
    fn write_rom(&mut self, _address: u16, _value: u8) {}
    fn read_ram(&self, address: u16) -> u8 {
        if let Some(ram) = &self.ram {
            ram[(address - EXTERNAL_RAM_START) as usize]
        } else {
            0xFF
        }
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        if let Some(ram) = &mut self.ram {
            ram[(address - EXTERNAL_RAM_START) as usize] = value;
        }
    }

    fn get_ram(&self) -> Option<&[u8]> {
        match &self.ram {
            Some(ram) => Some(ram),
            None => None,
        }
    }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
