use crate::{Cartridge, prelude::*};

/// addressable memory size
pub const ADDRESABLE_MEMORY: u16 = 0xFFFF; // 64KB
pub const ROM_BANK00_START: u16 = 0x0000;
// in DMG, in CGB 256 + 1792, splited in two parts, with the cartridge header in the middle
pub const BOOT_ROM_END: u16 = 0x0100;
pub const ROM_BANK00_END: u16 = 0x3FFF;
pub const ROM_BANKNN_START: u16 = 0x4000;
pub const ROM_BANKNN_END: u16 = 0x7FFF;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const EXTERNAL_RAM_START: u16 = 0xA000;
pub const EXTERNAL_RAM_END: u16 = 0xBFFF;
pub const WRAM_BANK0_START: u16 = 0xC000;
pub const WRAM_BANK0_END: u16 = 0xCFFF;
pub const WRAM_BANKN_START: u16 = 0xD000;
pub const WRAM_BANKN_END: u16 = 0xDFFF;
pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const NOT_USABLE_START: u16 = 0xFEA0;
pub const NOT_USABLE_END: u16 = 0xFEFF;
pub const IO_REGISTERS_START: u16 = 0xFF00;
pub const IO_REGISTERS_END: u16 = 0xFF7F;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const BOOT_REGISTER: u16 = 0xFF50;

pub fn is_high_address(address: u16) -> bool { address >= IO_REGISTERS_START && address <= ADDRESABLE_MEMORY }

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
///
/// __table from [Pan Docs](https://gbdev.io/pandocs/Memory_Map.html)__
///
/// Start       | End       | Description                                                       | Notes
/// ------------|-----------|------------------------------------------------------------------ |----------
/// 0000        | 3FFF      | 16 KiB ROM bank 00                                                | From cartridge, usually a fixed bank
/// 4000        | 7FFF      | 16 KiB ROM Bank 01–NN                                             | From cartridge, switchable bank via [mapper](#MBCs) (if any)
/// 8000        | 9FFF      | 8 KiB Video RAM (VRAM)                                            | In CGB mode, switchable bank 0/1
/// A000        | BFFF      | 8 KiB External RAM                                                | From cartridge, switchable bank if any
/// C000        | CFFF      | 4 KiB Work RAM (WRAM)                                             |
/// D000        | DFFF      | 4 KiB Work RAM (WRAM)                                             | In CGB mode, switchable bank 1–7
/// E000        | FDFF      | [Echo RAM](<#Echo RAM>) (mirror of C000–DDFF)                     | Prohibited
/// FE00        | FE9F      | [Object attribute memory (OAM)](<#Object Attribute Memory (OAM)>) |
/// FEA0        | FEFF      | [Not Usable](<#FEA0–FEFF range>)                                  | Prohibited
/// FF00        | FF7F      | [I/O Registers](<#I/O Ranges>)                                    |
/// FF80        | FFFE      | High RAM (HRAM)                                                   |
/// FFFF        | FFFF      | [Interrupt](#Interrupts) Enable register (IE)                     |
#[derive(Debug)]
pub struct Memory {
    pub game: Option<Cartridge>,
    pub boot_rom: Option<Vec<u8>>,

    pub rom: [u8; (ROM_BANKNN_END + 1) as usize],
    pub ram: [u8; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
    pub vram: [u8; (VRAM_END - VRAM_START + 1) as usize],
    pub external_ram: [u8; (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize],
    pub oam_ram: [u8; (OAM_END - OAM_START + 1) as usize],
    pub hram: [u8; (HRAM_END - HRAM_START + 1) as usize],
}

impl Memory {
    pub fn new(game: Option<Cartridge>, boot_rom: Option<Vec<u8>>) -> Memory {
        let mut rom = [0u8; (ROM_BANKNN_END as usize) + 1];

        // copy first from boot rom, and then from game
        // both initial copies are required in real hardware for nintendo logo check from boot rom and cartridge
        // used in real hardware to required games to have a nintendo logo in rom and allow nintendo to sue them if they're not allow (trademark violation)
        match (&game, &boot_rom) {
            (Some(game), Some(boot)) => {
                let boot_len = boot.len().min(BOOT_ROM_END as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);

                let game_len = game.rom.len().min((ROM_BANKNN_END + 1) as usize);
                rom[boot_len..game_len].copy_from_slice(&game.rom[boot_len..game_len]);
            }
            // copy only game if no boot rom is provided
            (Some(game), None) => {
                let game_len = game.rom.len().min(rom.len());
                rom[..game_len].copy_from_slice(&game.rom[..game_len]);
            }
            (None, Some(boot)) => {
                let boot_len = boot.len().min(BOOT_ROM_END as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);
            }
            _ => {}
        };

        Memory {
            game,
            boot_rom,
            rom,
            ram: [0; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
            vram: [0; (VRAM_END - VRAM_START + 1) as usize],
            external_ram: [0; (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize],
            oam_ram: [0; (OAM_END - OAM_START + 1) as usize],
            hram: [0; (HRAM_END - HRAM_START + 1) as usize],
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
            let game_len = game.rom.len().min((ROM_BANKNN_END + 1) as usize);
            self.rom[..game_len].copy_from_slice(&game.rom[..game_len]);
        }
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => &self.rom[address as usize],
            VRAM_START..=VRAM_END => &self.vram[(address - VRAM_START) as usize],
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                &self.external_ram[(address - EXTERNAL_RAM_START) as usize]
            }
            WRAM_BANK0_START..=WRAM_BANKN_END => &self.ram[(address - WRAM_BANK0_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                &self.ram[offset]
            }
            OAM_START..=OAM_END => &self.oam_ram[(address - OAM_START) as usize],
            NOT_USABLE_START..=NOT_USABLE_END => unreachable!(
                "Read to prohibited memory region [{}, {}] with address {:04X}",
                NOT_USABLE_START, NOT_USABLE_END, address
            ),
            HRAM_START..=HRAM_END => &self.hram[(address - HRAM_START) as usize],

            _ => unreachable!("Read of address {address:04X} should have been handled by other components"),
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => &mut self.rom[address as usize],
            VRAM_START..=VRAM_END => &mut self.vram[(address - VRAM_START) as usize],
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                &mut self.external_ram[(address - EXTERNAL_RAM_START) as usize]
            }
            WRAM_BANK0_START..=WRAM_BANKN_END => &mut self.ram[(address - WRAM_BANK0_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => &mut self.ram[(address - ECHO_RAM_START) as usize],
            OAM_START..=OAM_END => &mut self.oam_ram[(address - OAM_START) as usize],
            HRAM_START..=HRAM_END => &mut self.hram[(address - HRAM_START) as usize],

            _ => unreachable!("Read of address {address:04X} should have been handled by other components"),
        }
    }
}

impl Default for Memory {
    fn default() -> Self { Memory::new(None, None) }
}
