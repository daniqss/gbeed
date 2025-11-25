use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// Bitwise NOT, ComPLement accumulator.
/// Only operates on register A
pub struct Cpl {}

impl Cpl {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Cpl {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a = !gb.cpu.a;
        let flags = Flags {
            z: None,
            n: Some(true),
            h: Some(true),
            c: None,
        };

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "cpl") }
}
