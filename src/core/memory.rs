use std::{
    ops::{Deref, DerefMut, Index, IndexMut},
    rc::Rc,
};

use crate::{core::Cartridge, prelude::*};

/// addressable memory size
pub const ADDRESABLE_MEMORY: usize = 0xFFFF; // 64KB

pub struct MemoryBus {
    memory: [u8; ADDRESABLE_MEMORY],
}

impl MemoryBus {
    pub fn new(_cartrigde: Option<Rc<Cartridge>>) -> MemoryBus {
        let memory = [0; ADDRESABLE_MEMORY];

        // maybe cartridge should be copy here to memory, according to memory map

        MemoryBus { memory }
    }

    /// read 16 bits little endian word
    pub fn read_word(&self, address: u16) -> u16 {
        utils::to_u16(
            self.memory[address as usize],
            self.memory[(address + 1) as usize],
        )
    }

    /// write 16 bits little endian word
    pub fn write_word(&mut self, address: u16, value: u16) {
        let (high, low) = utils::to_u8(value);
        self.memory[address as usize] = high;
        self.memory[(address + 1) as usize] = low;
    }
}

impl Deref for MemoryBus {
    type Target = [u8];
    fn deref(&self) -> &Self::Target { &self.memory }
}

impl DerefMut for MemoryBus {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.memory }
}

impl Index<u16> for MemoryBus {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output { &self.memory[index as usize] }
}

impl IndexMut<u16> for MemoryBus {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output { &mut self.memory[index as usize] }
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
