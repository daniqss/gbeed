use std::fmt::Write;

use crate::core::cpu::{
    flags::{Flags, check_zero},
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

pub struct And<'a> {
    a: &'a mut u8,
    src: IT<'a>,
}

impl<'a> And<'a> {
    pub fn new(a: &'a mut u8, src: IT<'a>) -> Box<Self> { Box::new(Self { a, src }) }
}

impl<'a> Instruction<'a> for And<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (src, cycles, len): (u8, u8, u8) = match &self.src {
            IT::Register8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Immediate8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = *self.a & src;
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(true),
            c: Some(false),
        };
        *self.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "and a, {}", self.src)
    }
}
