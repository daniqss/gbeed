use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
    rc::Rc,
};

use crate::{core::ppu::Ppu, prelude::*};

/// addressable memory size
pub const ADDRESABLE_MEMORY: usize = 0xFFFF; // 64KB
pub const ROM_BANK00_START: u16 = 0x0000;
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
pub const INTERRUPT_ENABLE_REGISTER: u16 = 0xFFFF;

pub fn is_high_address(address: u16) -> bool {
    address >= IO_REGISTERS_START && address <= INTERRUPT_ENABLE_REGISTER
}

pub type MemoryBus = Rc<RefCell<Memory>>;

/// # Memory bus
/// different parts of the hardware access different parts of the memory map
/// This memory is distributed among the various hardware components
/// from this 16 bits address memory bus we can access all the memory mapped components
///
/// __table from [Pan Docs](https://gbdev.io/pandocs/Memory_Map.html)__
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
    pub game_rom: Option<Vec<u8>>,
    pub boot_rom: Option<Vec<u8>>,
    pub rom: [u8; (ROM_BANKNN_END + 1) as usize],
    pub ram: [u8; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
    pub vram: [u8; (VRAM_END - VRAM_START + 1) as usize],
    pub external_ram: [u8; (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize],
    pub oam_ram: [u8; (OAM_END - OAM_START + 1) as usize],
    pub io_registers: [u8; (IO_REGISTERS_END - IO_REGISTERS_START + 1) as usize],
    pub hram: [u8; (HRAM_END - HRAM_START + 1) as usize],
    pub interrupt_enable: u8,

    ppu: Option<Rc<RefCell<Ppu>>>,
}

impl Memory {
    pub fn new(
        game_rom: Option<Vec<u8>>,
        boot_rom: Option<Vec<u8>>,
        ppu: Option<Rc<RefCell<Ppu>>>,
    ) -> MemoryBus {
        let mut rom = [0u8; (ROM_BANKNN_END as usize) + 1];

        // copy first from boot rom, and then from game
        // both initial copies are required in real hardware for nintendo logo check from boot rom and cartridge
        // used in real hardware to required games to have a nintendo logo in rom and allow nintendo to sue them if they're not allow (trademark violation)
        match (&game_rom, &boot_rom) {
            (Some(game), Some(boot)) => {
                let boot_len = boot.len().min(BOOT_ROM_END as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);

                let game_len = game.len().min((ROM_BANKNN_END + 1) as usize);
                rom[boot_len..game_len].copy_from_slice(&game[boot_len..game_len]);
            }
            // copy only game if no boot rom is provided
            (Some(game), None) => {
                let game_len = game.len().min(rom.len());
                rom[..game_len].copy_from_slice(&game[..game_len]);
            }
            (None, Some(boot)) => {
                let boot_len = boot.len().min(BOOT_ROM_END as usize);
                rom[..boot_len].copy_from_slice(&boot[..boot_len]);
            }
            _ => {}
        };
        Rc::new(RefCell::new(Memory {
            game_rom,
            boot_rom,
            rom,
            ram: [0; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
            vram: [0; (VRAM_END - VRAM_START + 1) as usize],
            external_ram: [0; (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize],
            oam_ram: [0; (OAM_END - OAM_START + 1) as usize],
            io_registers: [0; (IO_REGISTERS_END - IO_REGISTERS_START + 1) as usize],
            hram: [0; (HRAM_END - HRAM_START + 1) as usize],
            interrupt_enable: 0,

            ppu,
        }))
    }

    /// read 16 bits little endian word
    pub fn read_word(&self, address: u16) -> u16 { utils::to_u16(self[address], self[address + 1]) }

    /// write 16 bits little endian word
    pub fn write_word(&mut self, address: u16, value: u16) {
        let (low, high) = utils::to_u8(value);
        self[address] = low;
        self[address + 1] = high;
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
                "Read to prohibited memory region [{}, {}]",
                NOT_USABLE_START, NOT_USABLE_END
            ),
            IO_REGISTERS_START..=IO_REGISTERS_END => {
                &self.io_registers[(address - IO_REGISTERS_START) as usize]
            }
            HRAM_START..=HRAM_END => &self.hram[(address - HRAM_START) as usize],
            INTERRUPT_ENABLE_REGISTER => &self.interrupt_enable,
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
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                &mut self.ram[offset]
            }
            OAM_START..=OAM_END => &mut self.oam_ram[(address - OAM_START) as usize],
            NOT_USABLE_START..=NOT_USABLE_END => unreachable!(
                "Write to prohibited memory region [{}, {}]",
                NOT_USABLE_START, NOT_USABLE_END
            ),
            IO_REGISTERS_START..=IO_REGISTERS_END => {
                &mut self.io_registers[(address - IO_REGISTERS_START) as usize]
            }
            HRAM_START..=HRAM_END => &mut self.hram[(address - HRAM_START) as usize],
            INTERRUPT_ENABLE_REGISTER => &mut self.interrupt_enable,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_byte() {
        let memory = Memory::new(None, None, None);
        memory.borrow_mut()[0x1234] = 0x56;
        assert_eq!(memory.borrow()[0x1234], 0x56);
    }

    #[test]
    fn test_read_write_word() {
        let memory = Memory::new(None, None, None);
        memory.borrow_mut().write_word(0x1234, 0x5678);
        assert_eq!(memory.borrow().read_word(0x1234), 0x5678);
        assert_eq!(memory.borrow()[0x1234], 0x78);
        assert_eq!(memory.borrow()[0x1235], 0x56);
    }
}
