use std::fmt::Write;

use super::InstructionTarget as IT;
use crate::{
    Dmg,
    core::cpu::{
        Reg,
        flags::{Flags, check_overflow_cy, check_overflow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
    },
};

/// Add with carry instruction
/// Adds the value of the specified target plus the carry flag to register A
pub struct Adc {
    carry: bool,
    addend: IT,
}

impl Adc {
    pub fn new(carry: bool, addend: IT) -> Box<Self> { Box::new(Adc { carry, addend }) }
}

impl Instruction for Adc {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (addend, cycles, len) = match &self.addend {
            IT::Reg8(val, reg) if *reg != Reg::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the addition
        // wrapping it prevent overflow panics in debug mode
        let mut result = gb.cpu.a.wrapping_add(addend);
        result = result.wrapping_add(if self.carry { 1 } else { 0 });

        // calculate flags
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(check_overflow_hc(result, gb.cpu.a)),
            c: Some(check_overflow_cy(result, gb.cpu.a)),
        };

        gb.cpu.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "adc a,{}", self.addend)
    }
}
