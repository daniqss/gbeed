use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R8,
    },
    prelude::*,
};

/// Sets bit u3 in register r8 to 0. Bit 0 is the rightmost one, bit 7 the leftmost one
pub struct ResR8 {
    bit: u8,
    dst: R8,
}
impl ResR8 {
    pub fn new(bit: u8, dst: R8) -> Box<Self> { Box::new(Self { bit, dst }) }
}
impl Instruction for ResR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8 & !(1 << self.bit);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("res {}, {}", self.bit, self.dst) }
}

pub struct ResPointedByHL {
    bit: u8,
}
impl ResPointedByHL {
    pub fn new(bit: u8) -> Box<Self> { Box::new(Self { bit }) }
}
impl Instruction for ResPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val & !(1 << self.bit);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("res {}, [hl]", self.bit) }
}

// Add a test module to match the pattern
#[cfg(test)]
mod tests {
    use super::*;

    use crate::cpu::R16;

    #[test]
    fn test_res_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1010_1010;
        let mut instr = ResR8::new(1, R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1010_1000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }

    #[test]
    fn test_res_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b1111_0000);
        let mut instr = ResPointedByHL::new(4);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b1110_0000);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }
}
