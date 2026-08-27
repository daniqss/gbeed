use crate::{
    cpu::{
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand16, WritableOperand},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Dec<D: WritableOperand> {
    dst: D,
}
impl<D: WritableOperand> Dec<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: WritableOperand> Instruction for Dec<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old = self.dst.read(gb);
        let result = old.wrapping_sub(1);
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(result)),
                n: Some(true),
                h: Some(check_borrow_hc(old, 1)),
                c: None,
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + D::READ_CYCLES + D::WRITE_CYCLES, 1 + D::LEN) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Dec16<D: Operand16> {
    dst: D,
}
impl<D: Operand16> Dec16<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: Operand16> Instruction for Dec16<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = self.dst.load(gb).wrapping_sub(1);
        self.dst.store(gb, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}
