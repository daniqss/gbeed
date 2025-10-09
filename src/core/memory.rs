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
    memory: [u8; ADDRESABLE_MEMORY],
}

impl MemoryBus {
    pub fn new(raw: Option<Vec<u8>>) -> MemoryBus {
        let mut memory = [0; ADDRESABLE_MEMORY];

        if let Some(raw) = raw {
            // copy ROM Bank 0
            let bank0_end = ROM_BANK00_END.min((raw.len() - 1) as u16);
            memory[ROM_BANK00_START as usize..=bank0_end as usize]
                .copy_from_slice(&raw[ROM_BANK00_START as usize..=bank0_end as usize]);

            // copy ROM Bank N (0x4000–0x7FFF) if exists, not sure yet how memory bank controllers work
            // for now I just copy it, games with 2 memory banks should work
            if raw.len() > ROM_BANKNN_START as usize {
                let bankn_end = ROM_BANKNN_END.min((raw.len() - 1) as u16);
                memory[ROM_BANKNN_START as usize..=bankn_end as usize]
                    .copy_from_slice(&raw[ROM_BANKNN_START as usize..=bankn_end as usize]);
            }
        }

        MemoryBus { memory }
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
    fn index(&self, index: u16) -> &Self::Output { &self.memory[index as usize] }
}

impl IndexMut<u16> for MemoryBus {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output { &mut self.memory[index as usize] }
}

impl Index<Range<u16>> for MemoryBus {
    type Output = [u8];
    fn index(&self, range: Range<u16>) -> &Self::Output {
        &self.memory[range.start as usize..range.end as usize]
    }
}

impl IndexMut<Range<u16>> for MemoryBus {
    fn index_mut(&mut self, range: Range<u16>) -> &mut Self::Output {
        &mut self.memory[range.start as usize..range.end as usize]
    }
}

impl Index<RangeInclusive<u16>> for MemoryBus {
    type Output = [u8];
    fn index(&self, range: RangeInclusive<u16>) -> &Self::Output {
        &self.memory[*range.start() as usize..=*range.end() as usize]
    }
}

impl IndexMut<RangeInclusive<u16>> for MemoryBus {
    fn index_mut(&mut self, range: RangeInclusive<u16>) -> &mut Self::Output {
        &mut self.memory[*range.start() as usize..=*range.end() as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_byte() {
        let mut memory = MemoryBus::new(None);
        memory[0x1234] = 0x56;
        assert_eq!(memory[0x1234], 0x56);
    }

    #[test]
    fn test_read_write_word() {
        let mut memory = MemoryBus::new(None);
        memory.write_word(0x1234, 0x5678);
        assert_eq!(memory.read_word(0x1234), 0x5678);
    }
}
