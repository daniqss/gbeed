use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand, WritableOperand},
    },
    prelude::*,
};

/// Load from/to high memory area instruction
/// Usually used to access memory mapped IO and HRAM,
/// so the used addresses are between 0xFF00 and 0xFFFF
#[derive(Debug, Default, Clone, Copy)]
pub struct Ldh<D: WritableOperand, S: Operand> {
    dst: D,
    src: S,
}
impl<D: WritableOperand, S: Operand> Ldh<D, S> {
    pub fn new(dst: D, src: S) -> Self { Self { dst, src } }
}
impl<D: WritableOperand, S: Operand> Instruction for Ldh<D, S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let value = self.src.read(gb);
        self.dst.write(gb, value);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES + D::WRITE_CYCLES, 1 + S::LEN + D::LEN) }
    fn disassembly(&self) -> String { format!("ldh {},{}", self.dst, self.src) }
}
