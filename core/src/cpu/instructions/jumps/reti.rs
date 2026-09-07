use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, stack},
    },
    prelude::*,
};

/// return from subroutine and enable interrupts
#[derive(Debug, Default, Clone, Copy)]
pub struct Reti;

impl Reti {
    pub fn new() -> Self { Self }
}

impl Instruction for Reti {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = true;

        gb.cpu.pc = stack::pop(gb);

        Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (4, 1) }
    fn disassembly(&self) -> String { "reti".to_string() }
}
