use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R8,
    },
    prelude::*,
};

/// Sets bit u3 in register r8 to 1. Bit 0 is the rightmost one, bit 7 the leftmost one
pub struct SetR8 {
    bit: u8,
    dst: R8,
}
impl SetR8 {
    pub fn new(bit: u8, dst: R8) -> Box<Self> { Box::new(Self { bit, dst }) }
}
impl Instruction for SetR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8 | (1 << self.bit);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("set {}, {}", self.bit, self.dst) }
}

pub struct SetPointedByHL {
    bit: u8,
}
impl SetPointedByHL {
    pub fn new(bit: u8) -> Box<Self> { Box::new(Self { bit }) }
}
impl Instruction for SetPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val | (1 << self.bit);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("set {}, [hl]", self.bit) }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::R16, Accessible16};

    use super::*;

    #[test]
    fn test_set_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0000;
        let mut instr = SetR8::new(1, R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0010);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }

    #[test]
    fn test_set_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0000);
        let mut instr = SetPointedByHL::new(4);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0001_0000);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }
}
