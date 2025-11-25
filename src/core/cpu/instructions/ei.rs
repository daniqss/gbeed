use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// enable interrupts
/// actually ime should be set AFTER the next instruction is executed
/// but let's hope it works the same
pub struct Ei {}

impl Ei {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Ei {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = true;
        Ok(InstructionEffect::new(1, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "ei") }
}
