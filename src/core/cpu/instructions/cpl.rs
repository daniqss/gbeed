use std::fmt::Write;

use crate::core::cpu::{
    flags::Flags,
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
        let flags = Flags {
            z: None,
            n: Some(true),
            h: Some(true),
            c: None,
        };

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "cpl") }
}
