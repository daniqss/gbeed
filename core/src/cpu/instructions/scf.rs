use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

pub struct Scf;

impl Scf {
    pub fn new() -> Box<Self> { Box::new(Self) }
}

impl Instruction for Scf {
    fn exec(&mut self, _gb: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: None,
                n: Some(false),
                h: Some(false),
                c: Some(true),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("scf") }
}
