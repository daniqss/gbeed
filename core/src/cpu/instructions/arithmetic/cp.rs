use crate::{
    cpu::{
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

/// ComPare instruction
/// Compare the value in register A with the given target
/// Works by subtract the src value from register A and sets flags accordingly, but does not store the result
#[derive(Debug, Default, Clone, Copy)]
pub struct Cp<S: Operand> {
    src: S,
}
impl<S: Operand> Cp<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for Cp<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = self.src.read(gb);
        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(result)),
                n: Some(true),
                h: Some(check_borrow_hc(gb.cpu.a, subtrahend)),
                c: Some(did_borrow),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("cp {}", self.src) }
}
