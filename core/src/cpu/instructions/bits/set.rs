use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

/// Sets bit u3 in any 8 bit operand to 1. Bit 0 is the rightmost one, bit 7 the leftmost one
#[derive(Debug, Default, Clone, Copy)]
pub struct Set<D: WritableOperand> {
    bit: u8,
    dst: D,
}
impl<D: WritableOperand> Set<D> {
    pub fn new(bit: u8, dst: D) -> Self { Self { bit, dst } }
}
impl<D: WritableOperand> Instruction for Set<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = self.dst.read(gb) | (1 << self.bit);
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("set {}, {}", self.bit, self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::{
        Accessible16,
        cpu::{R8, R16, instructions::PointedByHL},
    };

    use super::*;

    #[test]
    fn test_set_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0000;
        let mut instr = Set::new(1, R8::A);

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
        let mut instr = Set::new(4, PointedByHL);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0001_0000);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }
}
