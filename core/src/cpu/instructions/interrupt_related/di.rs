use crate::{
    cpu::instructions::{Instruction, InstructionEffect, InstructionResult},
    prelude::*,
};

/// disable interrupts
#[derive(Debug, Default, Clone, Copy)]
pub struct Di;

impl Di {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Di {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = false;
        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "di".to_string() }
}
