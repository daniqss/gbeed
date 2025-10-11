use std::ops::{Index, IndexMut, Range, RangeInclusive};

use crate::prelude::*;

/// addressable memory size
pub const ADDRESABLE_MEMORY: usize = 0xFFFF; // 64KB
pub const ROM_BANK00_START: u16 = 0x0000;
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
pub struct MemoryBus {
    pub game_rom: Option<Vec<u8>>,
    pub boot_rom: Option<Vec<u8>>,
    pub rom: [u8; (ROM_BANKNN_END + 1) as usize],
    pub ram: [u8; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
    pub vram: [u8; (VRAM_END - VRAM_START + 1) as usize],
}

impl MemoryBus {
    pub fn new(game_rom: Option<Vec<u8>>, boot_rom: Option<Vec<u8>>) -> MemoryBus {
        MemoryBus {
            game_rom: game_rom,
            boot_rom: boot_rom,
            rom: [0; (ROM_BANKNN_END as usize) + 1],
            ram: [0; (WRAM_BANKN_END - WRAM_BANK0_START + 1) as usize],
            vram: [0; (VRAM_END - VRAM_START + 1) as usize],
        }
    }

    /// read 16 bits little endian word
    pub fn read_word(&self, address: u16) -> u16 { utils::to_u16(self[address], self[address + 1]) }

    /// write 16 bits little endian word
    pub fn write_word(&mut self, address: u16, value: u16) {
        let (high, low) = utils::to_u8(value);
        self[address] = high;
        self[address + 1] = low;
    }
}

impl Index<u16> for MemoryBus {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => &self.rom[address as usize],
            VRAM_START..=VRAM_END => &self.vram[(address - VRAM_START) as usize],
            WRAM_BANK0_START..=WRAM_BANKN_END => &self.ram[(address - WRAM_BANK0_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                &self.ram[offset]
            }
            _ => &self.rom[0],
        }
    }
}

impl IndexMut<u16> for MemoryBus {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        match address {
            ROM_BANK00_START..=ROM_BANKNN_END => &mut self.rom[address as usize],
            VRAM_START..=VRAM_END => &mut self.vram[(address - VRAM_START) as usize],
            WRAM_BANK0_START..=WRAM_BANKN_END => {
                &mut self.ram[(address - WRAM_BANK0_START) as usize]
            }
            ECHO_RAM_START..=ECHO_RAM_END => {
                let offset = (address - ECHO_RAM_START) as usize;
                &mut self.ram[offset]
            }
            _ => &mut self.rom[0],
        }
    }
}

impl Index<Range<u16>> for MemoryBus {
    type Output = [u8];

    fn index(&self, range: Range<u16>) -> &Self::Output {
        let start = range.start;
        let end = range.end;

        match (start, end.saturating_sub(1)) {
            (ROM_BANK00_START..=ROM_BANKNN_END, ROM_BANK00_START..=ROM_BANKNN_END) => {
                &self.rom[start as usize..end as usize]
            }
            (VRAM_START..=VRAM_END, VRAM_START..=VRAM_END) => {
                let s = (start - VRAM_START) as usize;
                let e = (end - VRAM_START) as usize;
                &self.vram[s..e]
            }
            (WRAM_BANK0_START..=WRAM_BANKN_END, WRAM_BANK0_START..=WRAM_BANKN_END) => {
                let s = (start - WRAM_BANK0_START) as usize;
                let e = (end - WRAM_BANK0_START) as usize;
                &self.ram[s..e]
            }
            (ECHO_RAM_START..=ECHO_RAM_END, ECHO_RAM_START..=ECHO_RAM_END) => {
                let s = (start - ECHO_RAM_START) as usize;
                let e = (end - ECHO_RAM_START) as usize;
                &self.ram[s..e]
            }
            _ => &[],
        }
    }
}

impl IndexMut<Range<u16>> for MemoryBus {
    fn index_mut(&mut self, range: Range<u16>) -> &mut Self::Output {
        let start = range.start;
        let end = range.end;

        match (start, end.saturating_sub(1)) {
            (ROM_BANK00_START..=ROM_BANKNN_END, ROM_BANK00_START..=ROM_BANKNN_END) => {
                &mut self.rom[start as usize..end as usize]
            }
            (VRAM_START..=VRAM_END, VRAM_START..=VRAM_END) => {
                let s = (start - VRAM_START) as usize;
                let e = (end - VRAM_START) as usize;
                &mut self.vram[s..e]
            }
            (WRAM_BANK0_START..=WRAM_BANKN_END, WRAM_BANK0_START..=WRAM_BANKN_END) => {
                let s = (start - WRAM_BANK0_START) as usize;
                let e = (end - WRAM_BANK0_START) as usize;
                &mut self.ram[s..e]
            }
            (ECHO_RAM_START..=ECHO_RAM_END, ECHO_RAM_START..=ECHO_RAM_END) => {
                let s = (start - ECHO_RAM_START) as usize;
                let e = (end - ECHO_RAM_START) as usize;
                &mut self.ram[s..e]
            }
            _ => &mut [],
        }
    }
}

impl Index<RangeInclusive<u16>> for MemoryBus {
    type Output = [u8];

    fn index(&self, range: RangeInclusive<u16>) -> &Self::Output {
        let start = *range.start();
        let end = *range.end();

        match (start, end) {
            (ROM_BANK00_START..=ROM_BANKNN_END, ROM_BANK00_START..=ROM_BANKNN_END) => {
                &self.rom[start as usize..=end as usize]
            }
            (VRAM_START..=VRAM_END, VRAM_START..=VRAM_END) => {
                let s = (start - VRAM_START) as usize;
                let e = (end - VRAM_START) as usize;
                &self.vram[s..=e]
            }
            (WRAM_BANK0_START..=WRAM_BANKN_END, WRAM_BANK0_START..=WRAM_BANKN_END) => {
                let s = (start - WRAM_BANK0_START) as usize;
                let e = (end - WRAM_BANK0_START) as usize;
                &self.ram[s..=e]
            }
            (ECHO_RAM_START..=ECHO_RAM_END, ECHO_RAM_START..=ECHO_RAM_END) => {
                let s = (start - ECHO_RAM_START) as usize;
                let e = (end - ECHO_RAM_START) as usize;
                &self.ram[s..=e]
            }
            _ => &[],
        }
    }
}

impl IndexMut<RangeInclusive<u16>> for MemoryBus {
    fn index_mut(&mut self, range: RangeInclusive<u16>) -> &mut Self::Output {
        let start = *range.start();
        let end = *range.end();

        match (start, end) {
            (ROM_BANK00_START..=ROM_BANKNN_END, ROM_BANK00_START..=ROM_BANKNN_END) => {
                &mut self.rom[start as usize..=end as usize]
            }
            (VRAM_START..=VRAM_END, VRAM_START..=VRAM_END) => {
                let s = (start - VRAM_START) as usize;
                let e = (end - VRAM_START) as usize;
                &mut self.vram[s..=e]
            }
            (WRAM_BANK0_START..=WRAM_BANKN_END, WRAM_BANK0_START..=WRAM_BANKN_END) => {
                let s = (start - WRAM_BANK0_START) as usize;
                let e = (end - WRAM_BANK0_START) as usize;
                &mut self.ram[s..=e]
            }
            (ECHO_RAM_START..=ECHO_RAM_END, ECHO_RAM_START..=ECHO_RAM_END) => {
                let s = (start - ECHO_RAM_START) as usize;
                let e = (end - ECHO_RAM_START) as usize;
                &mut self.ram[s..=e]
            }
            _ => &mut [],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_byte() {
        let mut memory = MemoryBus::new(None, None);
        memory[0x1234] = 0x56;
        assert_eq!(memory[0x1234], 0x56);
    }

    #[test]
    fn test_read_write_word() {
        let mut memory = MemoryBus::new(None, None);
        memory.write_word(0x1234, 0x5678);
        assert_eq!(memory.read_word(0x1234), 0x5678);
    }
}
