use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

/// ComPare instruction
/// Compare the value in register A with the given target
/// Works by subtract the src value from register A and sets flags accordingly, but does not store the result
pub struct Cp {
    src: IT,
}

impl Cp {
    pub fn new(src: IT) -> Box<Self> { Box::new(Self { src }) }
}

impl Instruction for Cp {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (subtrahend, cycles, len): (u8, u8, u8) = match &self.src {
            IT::Reg8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(gb.cpu.a, subtrahend)),
            c: Some(did_borrow),
        };

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "or a, {}", self.src)
    }
}
