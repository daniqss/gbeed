use crate::prelude::*;

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

pub fn is_high_address(address: u16) -> bool { (IO_REGISTERS_START..=ADDRESABLE_MEMORY).contains(&address) }

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
    pub boot_rom: Option<Vec<u8>>,
    pub ram: Box<[u8; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize]>,
    pub hram: Box<[u8; HRAM_SIZE as usize]>,
}

impl Memory {
    pub fn new(boot_rom: Option<Vec<u8>>) -> Memory {
        Memory {
            boot_rom,
            ram: Box::new([0; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize]),
            hram: Box::new([0; HRAM_SIZE as usize]),
        }
    }
}

impl Default for Memory {
    fn default() -> Self { Memory::new(None) }
}

pub mod cgb {
    pub const KEY0_SYS: u16 = 0xFF4C;
    pub const KEY1_SPD: u16 = 0xFF4D;

    pub const VBK: u16 = 0xFF4F;

    pub const HDMA1: u16 = 0xFF51;
    pub const HDMA2: u16 = 0xFF52;
    pub const HDMA3: u16 = 0xFF53;
    pub const HDMA4: u16 = 0xFF54;
    pub const HDMA5: u16 = 0xFF55;

    pub const RP: u16 = 0xFF56;

    pub const BCPS_BGPI: u16 = 0xFF68;
    pub const BCPD_BGPD: u16 = 0xFF69;
    pub const OCPS_OBPI: u16 = 0xFF6A;
    pub const OCPD_OBPD: u16 = 0xFF6B;

    pub const OPRI: u16 = 0xFF6C;
    pub const SVBK_WBK: u16 = 0xFF70;

    pub const PCM12: u16 = 0xFF76;
    pub const PCM34: u16 = 0xFF77;
}
