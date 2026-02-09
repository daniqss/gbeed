use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// enable interrupts
/// actually ime should be set AFTER the next instruction is executed
/// but let's hope it works nevertheless
pub struct Ei {}

impl Ei {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Ei {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = true;
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("ei") }
}
