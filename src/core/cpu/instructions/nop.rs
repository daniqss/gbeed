use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// No OPeration
pub struct Nop;

impl Nop {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for Nop {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(self.info(gb), Flags::none()))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("nop") }
}
