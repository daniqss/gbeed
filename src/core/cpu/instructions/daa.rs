use crate::{
    Dmg,
    core::cpu::{
        flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// Decimal Adjust Accumulator. Meant to be used after an arithmetic operation whose input where in binary coded decimal.
pub struct Daa {}

impl Daa {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Daa {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let mut adjustament = 0;
        let mut carry = false;

        if (gb.cpu.f & SUBTRACTION_FLAG_MASK) != 0 {
            if gb.cpu.f & HALF_CARRY_FLAG_MASK != 0 {
                adjustament += 0x6;
            }
            if gb.cpu.f & CARRY_FLAG_MASK != 0 {
                adjustament += 0x60;
            }

            gb.cpu.a = gb.cpu.a.wrapping_sub(adjustament);
        } else {
            if (gb.cpu.f & HALF_CARRY_FLAG_MASK != 0) || (gb.cpu.a & 0x0F) > 0x09 {
                adjustament += 0x6;
            }
            if (gb.cpu.f & CARRY_FLAG_MASK != 0) || gb.cpu.a > 0x99 {
                adjustament += 0x60;
                carry = true;
            }

            gb.cpu.a = gb.cpu.a.wrapping_add(adjustament);
        }

        let flags = Flags {
            z: Some(gb.cpu.a == 0),
            n: None,
            h: Some(false),
            c: if carry { Some(true) } else { None },
        };

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "daa") }
}
