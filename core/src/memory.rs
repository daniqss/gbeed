use crate::{prelude::*, Cartridge};

/// addressable memory size, 64KB
pub const ADDRESABLE_MEMORY: u16 = 0xFFFF;

// in DMG, in CGB 256 + 1792, splited in two parts, with the cartridge header in the middle
mem_range!(BOOT_ROM, 0x0000, 0x00FF);

// From cartridge, usually a fixed bank
mem_range!(ROM_BANK00, 0x0000, 0x3FFF);

// From cartridge, switchable bank via [mapper](https://gbdev.io/pandocs/MBCs.html#mbcs) (if any)
mem_range!(ROM_BANKNN, 0x4000, 0x7FFF);

// In CGB mode, switchable bank 0/1
mem_range!(VRAM, 0x8000, 0x9FFF);

// From cartridge, switchable bank if any
mem_range!(EXTERNAL_RAM, 0xA000, 0xBFFF);

mem_range!(WRAM_BANK0, 0xC000, 0xCFFF);

// In CGB mode, switchable bank 1–7
mem_range!(WRAM_BANKN, 0xD000, 0xDFFF);

// Nintendo says use of this area is prohibited
mem_range!(ECHO_RAM, 0xE000, 0xFDFF);

mem_range!(OAM, 0xFE00, 0xFE9F);

// Nintendo says use of this area is prohibited
mem_range!(NOT_USABLE, 0xFEA0, 0xFEFF);

mem_range!(IO_REGISTERS, 0xFF00, 0xFF7F);

mem_range!(HRAM, 0xFF80, 0xFFFE);

pub const BOOT_REGISTER: u16 = 0xFF50;

pub fn is_high_address(address: u16) -> bool { address >= IO_REGISTERS_START && address <= ADDRESABLE_MEMORY }

/// # Memory mapped trait for addressable components
/// This trait allows to read and write from Dmg and its components, indexing it with a memory address
/// without the limitations of operators overloading traits
pub trait Accessible<Address8> {
    fn read(&self, address: Address8) -> u8;
    fn write(&mut self, address: Address8, value: u8);
}

pub trait Accessible16<Address16, Address8>: Accessible<Address8> {
    fn load(&self, address: Address16) -> u16;
    fn store(&mut self, address: Address16, value: u16);
}

/// # Memory bus
/// different parts of the hardware access different parts of the memory map
/// This memory is distributed among the various hardware components
/// from this 16 bits address memory bus we can access all the memory mapped components
#[derive(Debug)]
pub struct Memory {
    pub vram: [u8; VRAM_SIZE as usize],
    pub ram: [u8; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize],

    pub oam_ram: [u8; OAM_SIZE as usize],
    pub hram: [u8; HRAM_SIZE as usize],
}

impl Memory {
    pub fn new(game: &mut Cartridge, boot_rom: Option<Vec<u8>>) -> Memory {
        // copy first from boot rom, and then from game
        // both initial copies are required in real hardware for nintendo logo check from boot rom and cartridge
        // used in real hardware to required games to have a nintendo logo in rom and allow nintendo to sue them if they're not allow (trademark violation)
        if let Some(boot_rom) = boot_rom {
            let boot_len = boot_rom.len().min(BOOT_ROM_SIZE as usize);
            game.rom_bank00[..boot_len].copy_from_slice(&boot_rom[..boot_len]);
        }

        Memory {
            vram: [0; VRAM_SIZE as usize],
            ram: [0; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize],

            oam_ram: [0; OAM_SIZE as usize],
            hram: [0; HRAM_SIZE as usize],
        }
    }
}

impl Default for Memory {
    fn default() -> Self { Memory::new(&mut Cartridge::default(), None) }
}
