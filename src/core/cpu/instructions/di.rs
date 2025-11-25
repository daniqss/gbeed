use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// disable interrupts
pub struct Di {}

impl Di {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Di {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = false;
        Ok(InstructionEffect::new(1, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "di") }
}
