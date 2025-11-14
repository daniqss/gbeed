use std::fmt::Write;

use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// No OPeration
pub struct Nop;

impl Nop {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction<'_> for Nop {
    fn exec(&mut self) -> InstructionResult { Ok(InstructionEffect::new(1, 1, Flags::none())) }
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "nop") }
}
