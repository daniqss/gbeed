use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Decimal Adjust Accumulator. Meant to be used after an arithmetic operation whose input where in binary coded decimal.
#[derive(Debug, Default, Clone, Copy)]
pub struct Daa {
    carry: bool,
    half_carry: bool,
    subtraction: bool,
}

impl Daa {
    pub fn new(carry: bool, half_carry: bool, subtraction: bool) -> Self {
        Self {
            carry,
            half_carry,
            subtraction,
        }
    }
}

impl Instruction for Daa {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let mut adjustment = 0;
        let mut carry = false;

        if self.subtraction {
            if self.half_carry {
                adjustment += 0x6;
            }
            if self.carry {
                adjustment += 0x60;
            }

            gb.cpu.a = gb.cpu.a.wrapping_sub(adjustment);
        } else {
            if self.half_carry || (gb.cpu.a & 0x0F) > 0x09 {
                adjustment += 0x6;
            }
            if self.carry || gb.cpu.a > 0x99 {
                adjustment += 0x60;
                carry = true
            }

            gb.cpu.a = gb.cpu.a.wrapping_add(adjustment);
        }

        let flags = Flags {
            z: Some(gb.cpu.a == 0),
            n: None,
            h: Some(false),
            c: if carry { Some(true) } else { None },
        };

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "daa".to_string() }
}
