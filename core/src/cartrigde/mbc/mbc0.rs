use std::mem::MaybeUninit;

use crate::{
    BOOT_ROM_END, BOOT_ROM_START, EXTERNAL_RAM_SIZE, EXTERNAL_RAM_START, ROM_BANK00_SIZE, ROM_BANKNN_SIZE,
    cartrigde::{
        CartridgeError, CartridgeResult, RamSize, features::CartridgeFeatures, header::CartridgeHeader,
    },
};

use super::MemoryBankController;

const MBC0_ROM_SIZE: usize = (ROM_BANK00_SIZE + ROM_BANKNN_SIZE) as usize;
const MBC0_RAM_SIZE: usize = EXTERNAL_RAM_SIZE as usize;

/// Memory Bank Controller for cartridges without any MBC (ROM only mostly).
/// They can have a RAM chip using a discrete logic decode but without a full MCB.
#[derive(Debug)]
pub struct Mbc0 {
    // avoid 32kb stack allocation
    rom: Box<[u8; MBC0_ROM_SIZE]>,
    ram: Option<Box<[u8; MBC0_RAM_SIZE]>>,
}

impl MemoryBankController for Mbc0 {
    fn new(
        raw_rom: &[u8],
        save: Option<Vec<u8>>,
        features: &CartridgeFeatures,
        header: &CartridgeHeader,
    ) -> CartridgeResult<Self> {
        // SAFETY: store the ROM in the heap to avoid large stack allocation
        // that crash the emulator on some targets, like WASM
        // This way we avoid stack copy with compile time known size in heap allocation
        let rom: Box<[u8; MBC0_ROM_SIZE]> = unsafe {
            let mut boxed: Box<MaybeUninit<[u8; MBC0_ROM_SIZE]>> = Box::new(MaybeUninit::uninit());
            std::ptr::copy_nonoverlapping(raw_rom.as_ptr(), boxed.as_mut_ptr() as *mut u8, MBC0_ROM_SIZE);
            boxed.assume_init()
        };

        let ram: Option<Box<[u8; MBC0_RAM_SIZE]>> = match (features.has_ram, header.ram_size, save) {
            (true, RamSize::Ram8KB, Some(save_data)) => Some(unsafe {
                let mut boxed: Box<MaybeUninit<[u8; MBC0_RAM_SIZE]>> = Box::new(MaybeUninit::uninit());
                std::ptr::copy_nonoverlapping(
                    save_data.as_ptr(),
                    boxed.as_mut_ptr() as *mut u8,
                    MBC0_RAM_SIZE,
                );
                boxed.assume_init()
            }),
            (true, RamSize::Ram8KB, None) => Some(unsafe {
                let boxed: Box<MaybeUninit<[u8; MBC0_RAM_SIZE]>> = Box::new(MaybeUninit::zeroed());
                boxed.assume_init()
            }),
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

    fn get_ram(&self) -> Option<&[u8]> {
        match &self.ram {
            Some(ram) => Some(ram.as_slice()),
            None => None,
        }
    }
    fn swap_boot_rom(&mut self, boot_rom: &mut [u8]) {
        let rom_slice = &mut self.rom[BOOT_ROM_START as usize..=BOOT_ROM_END as usize];
        let boot_rom_slice = &mut boot_rom[..=(BOOT_ROM_END - BOOT_ROM_START) as usize];
        rom_slice.swap_with_slice(boot_rom_slice);
    }
}
