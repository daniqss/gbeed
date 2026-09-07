use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Swap<D: WritableOperand> {
    dst: D,
}
impl<D: WritableOperand> Swap<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: WritableOperand> Instruction for Swap<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = self.dst.read(gb).rotate_right(4);
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(result == 0),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("swap {}", self.dst) }
}
