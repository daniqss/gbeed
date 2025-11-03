use std::fmt::Write;

use super::InstructionTarget as IT;
use crate::core::cpu::{
    R8,
    flags::{CARRY_FLAG_MASK, check_carry, check_half_carry, check_zero},
    instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
};

/// Add with carry instruction
/// Adds the value of the specified target plus the carry flag to register A
pub struct ADC<'a> {
    a: &'a mut u8,
    f: &'a mut u8,
    addend: IT<'a>,
}

impl<'a> ADC<'a> {
    pub fn new(a: &'a mut u8, f: &'a mut u8, addend: IT<'a>) -> Box<Self> { Box::new(ADC { a, f, addend }) }
}

impl<'a> Instruction<'a> for ADC<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (addend, cycles, len) = match &self.addend {
            IT::Register8(val, reg) if *reg != R8::F => (val, 1, 1),
            IT::Immediate8(n8) => (n8, 2, 2),
            IT::PointedByHL(value) => (value, 2, 1),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the addition
        // wrapping it prevent overflow panics in debug mode
        let mut result = self.a.wrapping_add(*addend);
        result = result.wrapping_add(if (*self.f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 });
        let flags = check_zero(result) | check_carry(result, *self.a) | check_half_carry(result, *self.a);
        *self.a = result;

        Ok(InstructionEffect::new(len, cycles, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "adc a,{}", self.addend) }
}
