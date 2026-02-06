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
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("di") }
}
