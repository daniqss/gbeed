use std::fmt::Write;

use crate::core::cpu::instructions::{Instruction, InstructionEffect, InstructionResult};

/// No OPeration
pub struct NOP;

impl NOP {
    pub fn new() -> Box<Self> { Box::new(NOP) }
}

impl Instruction<'_> for NOP {
    fn exec(&mut self) -> InstructionResult { Ok(InstructionEffect::new(1, 1, None)) }
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "nop") }
}
