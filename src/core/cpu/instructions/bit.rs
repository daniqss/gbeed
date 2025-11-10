use crate::core::cpu::{
    R8,
    flags::{Flags, check_zero},
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

/// Test bit u3 in target
pub struct Bit<'a> {
    bit: u8,
    target: IT<'a>,
}

impl<'a> Bit<'a> {
    pub fn new(bit: u8, target: IT<'a>) -> Box<Self> { Box::new(Bit { bit, target }) }
}

impl<'a> Instruction<'a> for Bit<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (target, cycles, len): (u8, u8, u8) = match &self.target {
            IT::Register8(r8, reg) if *reg == R8::A => (*r8, 2, 2),
            IT::PointedByHL(val) => (*val, 3, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let test_bit = target & (1 << self.bit);
        let flags = Flags {
            z: Some(check_zero(test_bit)),
            n: Some(false),
            h: Some(true),
            c: None,
        };

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "bit {}, {}", self.bit, self.target)
    }
}
