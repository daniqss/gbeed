use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::{Flags, check_zero},
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

pub struct And {
    src: IT,
}

impl And {
    pub fn new(src: IT) -> Box<Self> { Box::new(Self { src }) }
}

impl<'a> Instruction for And {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (src, cycles, len): (u8, u8, u8) = match &self.src {
            IT::Reg8(r8, _) => (*r8, 1, 1),
            IT::PointedByHL(val) => (*val, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = gb.cpu.a & src;
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(true),
            c: Some(false),
        };
        gb.cpu.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "and a, {}", self.src)
    }
}
