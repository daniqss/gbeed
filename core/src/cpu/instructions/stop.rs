use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// In DGM is intended to enter a low power mode, but no licensed cartridge actually uses it
/// In GBC is used for speed switching
/// Because GBC support is not currently planned (at least now) we will just implement a Nop
/// This should be checked when lcd is implemented
pub struct Stop;

impl Stop {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for Stop {
    fn exec(&mut self, _: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("stop") }
}
