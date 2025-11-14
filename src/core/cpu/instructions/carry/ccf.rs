use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

pub struct Ccf {
    carry: bool,
}

impl Ccf {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}

impl Instruction<'_> for Ccf {
    fn exec(&mut self) -> InstructionResult {
        Ok(InstructionEffect::new(
            1,
            1,
            Flags {
                z: None,
                n: Some(false),
                h: Some(false),
                c: Some(!self.carry),
            },
        ))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "ccf") }
}
