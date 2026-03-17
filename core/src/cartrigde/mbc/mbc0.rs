use crate::{
    BOOT_ROM_END, BOOT_ROM_START, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANKNN_SIZE,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, RomSize, features::CartridgeFeatures,
        header::CartridgeHeader,
    },
};

use super::MemoryBankController;

const MBC0_ROM_SIZE: usize = (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize;
const MBC0_RAM_SIZE: usize = EXTERNAL_RAM_SIZE as usize;

/// Memory Bank Controller for cartridges without any MBC (ROM only mostly).
/// They can have a RAM chip using a discrete logic decode but without a full MCB.
#[derive(Debug)]
pub struct Mbc0 {
    rom: [u8; MBC0_ROM_SIZE],
    ram: Option<Vec<u8>>,
}

impl MemoryBankController for Mbc0 {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        let rom = match header.rom_size {
            RomSize::Rom32KB => raw_rom
                .get(..MBC0_ROM_SIZE)
                .and_then(|slice| slice.try_into().ok())
                .ok_or(CartridgeError::InvalidRomSize(
                    Some(header.rom_size),
                    "ROM size must be exactly 32KB for MBC0",
                ))?,
            _ => {
                return Err(CartridgeError::InvalidRomSize(
                    Some(header.rom_size),
                    "Only 32KB ROM size is supported for MBC0",
                ));
            }
        };

        let ram = match (features.has_ram, header.ram_size, save) {
            (true, RamSize::Ram8KB, Some(save_data)) => Some(save_data),
            (true, RamSize::Ram8KB, None) => Some(vec![0; MBC0_RAM_SIZE]),
            (false, RamSize::None, _) => None,
            (_, ram, _) => {
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

    fn get_ram(&self) -> Option<&[u8]> { self.ram.as_deref() }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
