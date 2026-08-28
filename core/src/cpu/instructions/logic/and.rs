use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct And<S: Operand> {
    src: S,
}
impl<S: Operand> And<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for And<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a &= self.src.read(gb);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(false),
                h: Some(true),
                c: Some(false),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("and {}", self.src) }
}
