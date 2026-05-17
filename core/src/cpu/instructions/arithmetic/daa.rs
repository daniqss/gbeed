use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, ZERO_FLAG_MASK, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Decimal Adjust Accumulator. Meant to be used after an arithmetic operation whose input where in binary coded decimal.
#[derive(Debug, Default, Clone, Copy)]
pub struct Daa;

impl Daa {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Daa {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let mut adjustment = 0;
        let mut carry = false;

        if gb.cpu.subtraction() {
            if gb.cpu.half_carry() {
                adjustment += 0x6;
            }
            if gb.cpu.carry() {
                adjustment += 0x60;
            }

            gb.cpu.a = gb.cpu.a.wrapping_sub(adjustment);
        } else {
            if gb.cpu.half_carry() || (gb.cpu.a & 0x0F) > 0x09 {
                adjustment += 0x6;
            }
            if gb.cpu.carry() || gb.cpu.a > 0x99 {
                adjustment += 0x60;
                carry = true
            }

            gb.cpu.a = gb.cpu.a.wrapping_add(adjustment);
        }

        Ok(InstructionEffect::new(
            self.info(),
            Some(DaaFlags::new(gb.cpu.a, carry).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "daa".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct DaaFlags {
    result: u8,
    carry: bool,
}

impl DaaFlags {
    fn new(result: u8, carry: bool) -> StaticBox<Self> { StaticBox::new(Self { result, carry }) }
}

impl LazyFlags for DaaFlags {
    fn updated_flags(&self) -> u8 {
        let base = ZERO_FLAG_MASK | HALF_CARRY_FLAG_MASK;
        if self.carry { base | CARRY_FLAG_MASK } else { base }
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { true }
}
