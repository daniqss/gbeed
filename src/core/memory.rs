use crate::prelude::*;

/// addressable memory size
pub const MEMORY_SIZE: usize = 0x10000; // 64KB

pub struct Memory {
    pub memory: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn read_word(&self, address: u16) -> u16 {
        utils::to_u16(
            self.memory[address as usize],
            self.memory[(address + 1) as usize],
        )
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let (high, low) = utils::to_u8(value);
        self.memory[address as usize] = high;
        self.memory[(address + 1) as usize] = low;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x1234, 0x56);
        assert_eq!(memory.read_byte(0x1234), 0x56);
        assert_eq!(memory.read_byte(0x1234), memory.memory[0x1234 as usize]);
    }

    #[test]
    fn test_read_write_word() {
        let mut memory = Memory::new();
        memory.write_word(0x1234, 0x5678);
        assert_eq!(memory.read_word(0x1234), 0x5678);
        assert_eq!(
            memory.read_word(0x1234),
            utils::to_u16(
                memory.memory[0x1234 as usize],
                memory.memory[0x1235 as usize]
            )
        );
    }
}
