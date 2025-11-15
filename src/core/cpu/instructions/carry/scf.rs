use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

pub struct Scf;

impl Scf {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction<'_> for Scf {
    fn exec(&mut self) -> InstructionResult {
        Ok(InstructionEffect::new(
            1,
            1,
            Flags {
                z: None,
                n: Some(false),
                h: Some(false),
                c: Some(true),
            },
        ))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "scf") }
}
