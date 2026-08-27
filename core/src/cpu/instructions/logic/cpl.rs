use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Bitwise NOT, ComPLement accumulator.
/// Only operates on register A
#[derive(Debug, Default, Clone, Copy)]
pub struct Cpl;

impl Cpl {
    pub fn new() -> Self { Self }
}

impl Instruction for Cpl {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a = !gb.cpu.a;

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: None,
                n: Some(true),
                h: Some(true),
                c: None,
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "cpl".to_string() }
}
