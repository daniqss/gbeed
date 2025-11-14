use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// disable interrupts
pub struct Di<'a> {
    ime: &'a mut bool,
}

impl<'a> Di<'a> {
    pub fn new(ime: &'a mut bool) -> Box<Self> { Box::new(Self { ime }) }
}

impl<'a> Instruction<'a> for Di<'a> {
    fn exec(&mut self) -> InstructionResult {
        *self.ime = false;
        Ok(InstructionEffect::new(1, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "di") }
}
