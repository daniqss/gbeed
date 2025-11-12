use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// TODO: This should be implemented with interruptions management
pub struct Ei;

impl Ei {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction<'_> for Ei {
    fn exec(&mut self) -> InstructionResult { Ok(InstructionEffect::new(2, 2, Flags::none())) }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "ei") }
}
