use crate::core::cpu::{
    flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK},
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// Decimal Adjust Accumulator. Meant to be used after an arithmetic operation whose input where in binary coded decimal.
pub struct Daa<'a> {
    a: &'a mut u8,
    f: &'a mut u8,
}

impl<'a> Daa<'a> {
    pub fn new(a: &'a mut u8, f: &'a mut u8) -> Box<Self> { Box::new(Self { a, f }) }
}

impl<'a> Instruction<'a> for Daa<'a> {
    fn exec(&mut self) -> InstructionResult {
        let mut adjustament = 0;
        let mut carry = false;

        if (*self.f & SUBTRACTION_FLAG_MASK) != 0 {
            if *self.f & HALF_CARRY_FLAG_MASK != 0 {
                adjustament += 0x6;
            }
            if *self.f & CARRY_FLAG_MASK != 0 {
                adjustament += 0x60;
            }

            *self.a = self.a.wrapping_sub(adjustament);
        } else {
            if (*self.f & HALF_CARRY_FLAG_MASK != 0) || (*self.a & 0x0F) > 0x09 {
                adjustament += 0x6;
            }
            if (*self.f & CARRY_FLAG_MASK != 0) || *self.a > 0x99 {
                adjustament += 0x60;
                carry = true;
            }

            *self.a = self.a.wrapping_add(adjustament);
        }

        let flags = Flags {
            z: Some(*self.a == 0),
            n: None,
            h: Some(false),
            c: if carry { Some(true) } else { None },
        };

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "daa") }
}
