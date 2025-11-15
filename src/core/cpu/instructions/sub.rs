use std::fmt::Write;

use crate::core::cpu::{
    R8,
    flags::{Flags, check_borrow_hc, check_zero},
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

/// Subtraction instruction
/// Subtracts the value of the specified target from register A
/// Always sets the subtraction flag, sets zero flag if result is zero, and sets half-carry and carry flags if there is a borrow in bits 4
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
            IT::Reg8(val, reg) if *reg != R8::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the subtraction
        let (result, did_borrow) = self.a.overflowing_sub(subtrahend);

        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(*self.a, subtrahend)),
            c: Some(did_borrow),
        };

        *self.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "sub a,{}", self.subtrahend)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_sub_zero_result() {
        let mut a = 20;
        let subtrahend = IT::Imm8(20);
        let mut instr = Sub::new(&mut a, subtrahend);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(true),
                h: Some(false),
                c: Some(false),
            }
        );
    }

    #[test]
    fn test_sub_set_half_carry() {
        let mut a = 0b0001_0000;
        let subtrahend = IT::Reg8(0b0000_0001, R8::B);

        let mut instr = Sub::new(&mut a, subtrahend);
        let result = instr.exec().unwrap();

        assert_eq!(a, 0x0F);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(true),
                c: Some(false),
            }
        );
    }

    #[test]
    fn test_sub_set_carry() {
        let mut a = 0x10;
        let subtrahend = IT::PointedByHL(0x20);

        let mut instr = Sub::new(&mut a, subtrahend);
        let result = instr.exec().unwrap();

        assert_eq!(a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(true),
            }
        );
    }
}
