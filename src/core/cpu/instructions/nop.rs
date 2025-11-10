use std::fmt::Write;

use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// No OPeration
pub struct NOP;

impl NOP {
    pub fn new() -> Box<Self> { Box::new(NOP) }
}

impl Instruction<'_> for NOP {
    fn exec(&mut self) -> InstructionResult { Ok(InstructionEffect::new(1, 1, Flags::none())) }
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "nop") }
}
