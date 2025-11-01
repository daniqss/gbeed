use std::fmt::Write;

use super::InstructionTarget as IT;
use crate::core::cpu::{
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
    pub fn new(a: &'a mut u8, f: &'a mut u8, addend: IT<'a>) -> Self { ADC { a, f, addend } }
}

impl<'a> Instruction<'a> for ADC<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (addend, cycles, len) = match self.addend {
            IT::Immediate(n8) => (n8, 2, 2),
            IT::RegisterB(b) => (b, 1, 1),
            IT::RegisterC(c) => (c, 1, 1),
            IT::RegisterD(d) => (d, 1, 1),
            IT::RegisterE(e) => (e, 1, 1),
            IT::RegisterH(h) => (h, 1, 1),
            IT::RegisterL(l) => (l, 1, 1),
            IT::PointedByHL(value) => (value, 2, 1),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the addition
        // wrapping it prevent overflow panics in debug mode
        let mut result = self.a.wrapping_add(addend);
        result = result.wrapping_add(if (*self.f & CARRY_FLAG_MASK) != 0 {
            1
        } else {
            0
        });
        let flags =
            check_zero(result) | check_carry(result, *self.a) | check_half_carry(result, *self.a);

        Ok(InstructionEffect::new(len, cycles, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), InstructionError> {
        write!(
            w,
            "adc a,{}",
            match self.addend {
                IT::Immediate(n8) => format!("{}", n8),
                IT::RegisterB(_) => "b".to_string(),
                IT::RegisterC(_) => "c".to_string(),
                IT::RegisterD(_) => "d".to_string(),
                IT::RegisterE(_) => "e".to_string(),
                IT::RegisterH(_) => "h".to_string(),
                IT::RegisterL(_) => "l".to_string(),
                IT::PointedByHL(_) => "[hl]".to_string(),
                _ => return Err(InstructionError::MalformedInstruction),
            }
        )
        .map_err(|_| InstructionError::MalformedInstruction)
    }
}
