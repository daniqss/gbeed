use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// No OPeration
pub struct Nop;

impl Nop {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for Nop {
    fn exec(&mut self, _: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("nop") }
}
