use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

pub struct Ccf {
    carry: bool,
}

impl Ccf {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}

impl Instruction for Ccf {
    fn exec(&mut self, _gb: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: None,
                n: Some(false),
                h: Some(false),
                c: Some(!self.carry),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("ccf") }
}
