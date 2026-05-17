use crate::{
    cpu::instructions::{Instruction, InstructionEffect, InstructionResult},
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Halt;

impl Halt {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Halt {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.halted = true;
        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "halt".to_string() }
}
