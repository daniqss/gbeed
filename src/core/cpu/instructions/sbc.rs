use std::fmt::Write;

use super::InstructionTarget as IT;
use crate::core::cpu::{
    R8,
    flags::{CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, check_borrow_cy, check_borrow_hc, check_zero},
    instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
};

/// Substraction with carry instruction
pub struct Sbc<'a> {
    a: &'a mut u8,
    f: &'a mut u8,
    subtrahend: IT<'a>,
}

impl<'a> Sbc<'a> {
    pub fn new(a: &'a mut u8, f: &'a mut u8, subtrahend: IT<'a>) -> Box<Self> {
        Box::new(Sbc { a, f, subtrahend })
    }
}

impl<'a> Instruction<'a> for Sbc<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (subtrahend, cycles, len) = match &self.subtrahend {
            IT::Register8(val, reg) if *reg != R8::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Immediate8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform substraction
        let (result, did_borrow_sub) = self.a.overflowing_sub(subtrahend);
        let (result, did_borrow_cy) =
            result.overflowing_sub(if (*self.f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 });

        // calculate new flags
        let flags = check_zero(result)
            | SUBTRACTION_FLAG_MASK
            | check_borrow_hc(*self.a, subtrahend)
            | check_borrow_cy(did_borrow_sub || did_borrow_cy);

        *self.a = result;

        Ok(InstructionEffect::new(cycles, len, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "sbc a,{}", self.subtrahend)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::{HALF_CARRY_FLAG_MASK, ZERO_FLAG_MASK};

    use super::*;

    #[test]
    fn test_sbc_zero_result() {
        let mut a = 20;
        let mut f = CARRY_FLAG_MASK;
        let subtrahend = IT::Immediate8(19);

        let mut sbc = Sbc::new(&mut a, &mut f, subtrahend);
        let result = sbc.exec().unwrap();

        assert_eq!(a, 0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(result.flags.unwrap(), ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK);
        assert_eq!(result.flags.unwrap() & HALF_CARRY_FLAG_MASK, 0);
        assert_eq!(result.flags.unwrap() & CARRY_FLAG_MASK, 0);
    }

    #[test]
    fn test_sbc_set_half_carry() {
        let mut a = 0b0001_0000;
        let mut f = 0;
        let subtrahend = IT::Register8(0b0000_00011, R8::B);

        let mut sbc = Sbc::new(&mut a, &mut f, subtrahend);
        let result = sbc.exec().unwrap();

        assert_eq!(a, 0b0000_1101);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags.unwrap(),
            HALF_CARRY_FLAG_MASK | SUBTRACTION_FLAG_MASK
        );
        assert_eq!(result.flags.unwrap() & ZERO_FLAG_MASK, 0);
        assert_eq!(result.flags.unwrap() & CARRY_FLAG_MASK, 0);
    }

    #[test]
    fn test_sbc_set_carry() {
        let mut a = 0x10;
        let mut f = 0;
        let subtrahend = IT::PointedByHL(0x20);

        let mut sbc = Sbc::new(&mut a, &mut f, subtrahend);
        let result = sbc.exec().unwrap();

        assert_eq!(a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 1);
        assert_eq!(result.flags.unwrap(), CARRY_FLAG_MASK | SUBTRACTION_FLAG_MASK);
        assert_eq!(result.flags.unwrap() & ZERO_FLAG_MASK, 0);
        assert_eq!(result.flags.unwrap() & HALF_CARRY_FLAG_MASK, 0);
    }
}
