use std::fmt::Write;

use crate::core::cpu::{
    flags::{SUBTRACTION_FLAG_MASK, check_borrow_cy, check_borrow_hc, check_zero},
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

/// ComPare instruction
/// Compare the value in register A with the given target
/// Works by subtract the src value from register A and sets flags accordingly, but does not store the result
pub struct Cp<'a> {
    a: u8,
    src: IT<'a>,
}

impl<'a> Cp<'a> {
    pub fn new(a: u8, src: IT<'a>) -> Box<Self> { Box::new(Self { a, src }) }
}

impl<'a> Instruction<'a> for Cp<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (subtrahend, cycles, len): (u8, u8, u8) = match &self.src {
            IT::Register8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Immediate8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let (result, did_borrow) = self.a.overflowing_sub(subtrahend);
        let flags = check_zero(result)
            | SUBTRACTION_FLAG_MASK
            | check_borrow_hc(self.a, subtrahend)
            | check_borrow_cy(did_borrow);

        Ok(InstructionEffect::new(cycles, len, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "or a, {}", self.src)
    }
}
