use std::fmt::Write;

use crate::core::cpu::{
    flags::{Flags, check_borrow_hc, check_zero},
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
            IT::Reg8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let (result, did_borrow) = self.a.overflowing_sub(subtrahend);
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(self.a, subtrahend)),
            c: Some(did_borrow),
        };

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "or a, {}", self.src)
    }
}
