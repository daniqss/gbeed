use crate::prelude::*;

/// addressable memory size
pub const ADDRESABLE_MEMORY: usize = 0xFFFF; // 64KB

pub struct MemoryBus {
    memory: [u8; ADDRESABLE_MEMORY],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; ADDRESABLE_MEMORY],
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
        let mut memory = MemoryBus::new();
        memory.write_byte(0x1234, 0x56);
        assert_eq!(memory.read_byte(0x1234), 0x56);
        assert_eq!(memory.read_byte(0x1234), memory.memory[0x1234 as usize]);
    }

    #[test]
    fn test_read_write_word() {
        let mut memory = MemoryBus::new();
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
