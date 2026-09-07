use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

/// Test bit u3 for any 8 bit operand
#[derive(Debug, Default, Clone, Copy)]
pub struct Bit<S: Operand> {
    bit: u8,
    target: S,
}
impl<S: Operand> Bit<S> {
    pub fn new(bit: u8, target: S) -> Self { Self { bit, target } }
}
impl<S: Operand> Instruction for Bit<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let test_bit = self.target.read(gb) & (1 << self.bit);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(test_bit)),
                n: Some(false),
                h: Some(true),
                c: None,
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2 + S::READ_CYCLES, 2 + S::LEN) }
    fn disassembly(&self) -> String { format!("bit {}, {}", self.bit, self.target) }
}
