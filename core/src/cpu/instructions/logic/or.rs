use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Or<S: Operand> {
    src: S,
}
impl<S: Operand> Or<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for Or<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a |= self.src.read(gb);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("or {}", self.src) }
}
