use crate::{
    Dmg,
    core::cpu::{
        flags::{Flags, check_zero},
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

/// Test bit u3 in target
pub struct Bit {
    bit: u8,
    target: IT,
}

impl Bit {
    pub fn new(bit: u8, target: IT) -> Box<Self> { Box::new(Bit { bit, target }) }
}

impl Instruction for Bit {
    fn exec(&mut self, _gb: &mut Dmg) -> InstructionResult {
        let (target, cycles, len): (u8, u8, u8) = match &self.target {
            IT::Reg8(r8, _) => (*r8, 2, 2),
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
