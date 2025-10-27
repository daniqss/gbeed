use super::InstructionTarget as IT;
use crate::core::cpu::{
    flags::{CARRY_FLAG_MASK, check_carry, check_half_carry, check_zero},
    instructions::{InstructionEffect, InstructionError, InstructionResult},
};

/// Add with carry instruction
///
pub struct ADC<'a> {
    addend: IT<'a>,
}

impl<'a> ADC<'a> {
    pub fn new(addend: IT<'a>) -> Self { ADC { addend } }

    pub fn exec(addend: IT, f: u8, a: &mut u8) -> InstructionResult {
        let (addend, cycles, len) = match addend {
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
        let mut result = a.wrapping_add(addend);
        result = result.wrapping_add(if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 });
        let flags = check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a);

        Ok(InstructionEffect::new(len, cycles, Some(flags)))
    }
}

impl std::fmt::Display for ADC<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ADC A,{}",
            match self.addend {
                IT::Immediate(n8) => format!("{}", n8),
                IT::RegisterB(_) => "B".to_string(),
                IT::RegisterC(_) => "C".to_string(),
                IT::RegisterD(_) => "D".to_string(),
                IT::RegisterE(_) => "E".to_string(),
                IT::RegisterH(_) => "H".to_string(),
                IT::RegisterL(_) => "L".to_string(),
                IT::PointedByHL(_) => "[HL]".to_string(),
                _ => unreachable!("ADC instruction doesn't support target {:?}", self.addend),
            }
        )
    }
}
