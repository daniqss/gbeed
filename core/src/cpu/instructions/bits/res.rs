use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

/// Sets bit u3 in any 8 bit operand to 0. Bit 0 is the rightmost one, bit 7 the leftmost one
#[derive(Debug, Default, Clone, Copy)]
pub struct Res<D: WritableOperand> {
    bit: u8,
    dst: D,
}
impl<D: WritableOperand> Res<D> {
    pub fn new(bit: u8, dst: D) -> Self { Self { bit, dst } }
}
impl<D: WritableOperand> Instruction for Res<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = self.dst.read(gb) & !(1 << self.bit);
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("res {}, {}", self.bit, self.dst) }
}

// Add a test module to match the pattern
#[cfg(test)]
mod tests {
    use crate::cpu::{R8, R16, instructions::PointedByHL};

    use super::*;

    #[test]
    fn test_res_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1010_1010;
        let mut instr = Res::new(1, R8::A);

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
        let mut instr = Res::new(4, PointedByHL);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b1110_0000);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        assert_eq!(result.flags, Flags::none());
    }
}
