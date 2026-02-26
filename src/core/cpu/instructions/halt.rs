use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// TODO: This should be implemented with interruptions management
pub struct Halt;

impl Halt {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for Halt {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.halted = true;
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("halt") }
}
