use std::fmt::Write;

use crate::core::cpu::{
    R8,
    flags::{SUBTRACTION_FLAG_MASK, check_borrow_cy, check_borrow_hc, check_zero},
    instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT, sub},
};

pub struct Sub<'a> {
    a: &'a mut u8,
    subtrahend: IT<'a>,
}

impl<'a> Sub<'a> {
    pub fn new(a: &'a mut u8, subtrahend: IT<'a>) -> Box<Self> { Box::new(Sub { a, subtrahend }) }
}

impl<'a> Instruction<'a> for Sub<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (subtrahend, cycles, len) = match &self.subtrahend {
            IT::Register8(val, reg) if *reg != R8::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Immediate8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the subtraction
        let result = self.a.wrapping_sub(subtrahend);

        // calculate flags
        let flags = check_zero(result)
            | SUBTRACTION_FLAG_MASK
            | check_borrow_hc(*self.a, subtrahend)
            | check_borrow_cy(*self.a, subtrahend);

        *self.a = result;

        Ok(InstructionEffect::new(len, cycles, Some(flags)))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "sub a,{}", self.subtrahend) }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, ZERO_FLAG_MASK};

    use super::*;

    #[test]
    fn test_sub_zero_result() {
        let mut a = 20;
        let subtrahend = IT::Immediate8(20);
        let mut instr = Sub::new(&mut a, subtrahend);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(result.flags.unwrap(), ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK);
    }

    #[test]
    fn test_sub_set_half_carry() {
        let mut a = 0b0001_0000;
        let subtrahend = IT::Register8(0b0000_0001, R8::B);
        let mut instr = Sub::new(&mut a, subtrahend);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0x0F);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags.unwrap(),
            CARRY_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK
        );
    }
}
