use std::fmt::Write;

use crate::core::cpu::{
    flags::{HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK},
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// Bitwise NOT, ComPLement accumulator.
/// Only operates on register A
pub struct Cpl<'a> {
    a: &'a mut u8,
}

impl<'a> Cpl<'a> {
    pub fn new(a: &'a mut u8) -> Box<Self> { Box::new(Self { a }) }
}

impl<'a> Instruction<'a> for Cpl<'a> {
    fn exec(&mut self) -> InstructionResult {
        *self.a = !*self.a;
        let flags = SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK;

        Ok(InstructionEffect::new(1, 1, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "cpl") }
}
