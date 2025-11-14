use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// enable interrupts
/// actually ime should be set AFTER the next instruction is executed
/// but let's hope it works the same
pub struct Ei<'a> {
    ime: &'a mut bool,
}

impl<'a> Ei<'a> {
    pub fn new(ime: &'a mut bool) -> Box<Self> { Box::new(Self { ime }) }
}

impl<'a> Instruction<'a> for Ei<'a> {
    fn exec(&mut self) -> InstructionResult {
        *self.ime = true;
        Ok(InstructionEffect::new(1, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "ei") }
}
