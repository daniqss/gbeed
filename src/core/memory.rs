use crate::{Cartridge, mem_range, prelude::*};

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
pub trait MemoryMapped {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read16(&self, address: u16) -> u16 {
        let low = self.read(address) as u16;
        let high = self.read(address.wrapping_add(1)) as u16;

        (high << 8) | low
    }

    fn write16(&mut self, address: u16, value: u16) {
        let low = (value & 0x00FF) as u8;
        let high = (value >> 8) as u8;

        self.write(address, low);
        self.write(address.wrapping_add(1), high);
    }
}

/// # Memory mapped trait for addressable components
/// This trait allows to read and write from Dmg and its components, indexing it with a memory address or a Cpu register
pub trait Accessable<Address8, Address16>: Index<Address8, Output = u8> + IndexMut<Address8> {
    fn read16(&self, addr: Address16) -> u16;
    fn write16(&mut self, addr: Address16, value: u16);
}

/// # Memory bus
/// different parts of the hardware access different parts of the memory map
/// This memory is distributed among the various hardware components
/// from this 16 bits address memory bus we can access all the memory mapped components
#[derive(Debug)]
pub struct Memory {
    pub game: Option<Cartridge>,
    pub boot_rom: Option<Vec<u8>>,

    pub rom: [u8; (ROM_BANKNN_SIZE + ROM_BANK00_SIZE) as usize],
    pub vram: [u8; VRAM_SIZE as usize],
    pub external_ram: [u8; EXTERNAL_RAM_SIZE as usize],
    pub ram: [u8; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize],

    pub oam_ram: [u8; OAM_SIZE as usize],
    pub hram: [u8; HRAM_SIZE as usize],
}

impl Memory {
    pub fn new(game: Option<Cartridge>, boot_rom: Option<Vec<u8>>) -> Memory {
        let mut rom = [0u8; (ROM_BANKNN_SIZE + ROM_BANK00_SIZE) as usize];

        // copy first from boot rom, and then from game
        // both initial copies are required in real hardware for nintendo logo check from boot rom and cartridge
        // used in real hardware to required games to have a nintendo logo in rom and allow nintendo to sue them if they're not allow (trademark violation)
        match (&game, &boot_rom) {
            (Some(game), Some(boot)) => {
                let boot_len = boot.len().min((ROM_BANKNN_SIZE - 1) as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);

                let game_len = game.rom.len().min(ROM_BANKNN_SIZE as usize);
                rom[boot_len..game_len].copy_from_slice(&game.rom[boot_len..game_len]);
            }
            // copy only game if no boot rom is provided
            (Some(game), None) => {
                let game_len = game.rom.len().min(rom.len());
                rom[..game_len].copy_from_slice(&game.rom[..game_len]);
            }
            (None, Some(boot)) => {
                let boot_len = boot.len().min((ROM_BANKNN_SIZE - 1) as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);
            }
            _ => {}
        };

        Memory {
            game,
            boot_rom,

            rom,
            vram: [0; VRAM_SIZE as usize],
            external_ram: [0; EXTERNAL_RAM_SIZE as usize],
            ram: [0; (WRAM_BANKN_SIZE + WRAM_BANK0_SIZE) as usize],

            oam_ram: [0; OAM_SIZE as usize],
            hram: [0; HRAM_SIZE as usize],
        }
    }

    /// unmaps boot rom when boot reaches pc = 0x00FE, when load 1 in bank register (0xFF50)
    /// ```asm
    /// ld a, $01
    /// ld [0xFF50], a
    /// ```
    /// Next instruction will be the first `nop` in 0x0100, in the cartridge rom
    pub fn unmap_boot_rom(&mut self) {
        if let Some(game) = &self.game {
            println!("Unmapping boot rom, switching to cartridge rom");
            let game_len = game.rom.len().min((ROM_BANKNN_END + 1) as usize);
            self.rom[..game_len].copy_from_slice(&game.rom[..game_len]);
        }
    }
}

impl Default for Memory {
    fn default() -> Self { Memory::new(None, None) }
}
