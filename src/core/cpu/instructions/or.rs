use std::fmt::Write;

use crate::core::cpu::{
    flags::check_zero,
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

pub struct Or<'a> {
    a: &'a mut u8,
    src: IT<'a>,
}

impl<'a> Or<'a> {
    pub fn new(a: &'a mut u8, src: IT<'a>) -> Box<Self> { Box::new(Self { a, src }) }
}

impl<'a> Instruction<'a> for Or<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (src, cycles, len): (u8, u8, u8) = match &self.src {
            IT::Register8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Immediate8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = *self.a | src;
        let flags = check_zero(result);
        *self.a = result;

        Ok(InstructionEffect::new(cycles, len, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "or a, {}", self.src)
    }
}
