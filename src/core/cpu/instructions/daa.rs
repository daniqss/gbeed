use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
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

        if gb.cpu.substraction() {
            if gb.cpu.half_carry() {
                adjustament += 0x6;
            }
            if gb.cpu.carry() {
                adjustament += 0x60;
            }

            gb.cpu.a = gb.cpu.a.wrapping_sub(adjustament);
        } else {
            if gb.cpu.half_carry() || (gb.cpu.a & 0x0F) > 0x09 {
                adjustament += 0x6;
            }
            if gb.cpu.carry() || gb.cpu.a > 0x99 {
                adjustament += 0x60;
                carry = true
            }

            gb.cpu.a = gb.cpu.a.wrapping_add(adjustament);
        }

        let flags = Flags {
            z: Some(gb.cpu.a == 0),
            n: None,
            h: Some(false),
            c: if carry { Some(true) } else { None },
        };

        Ok(InstructionEffect::new(self.info(gb), flags))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("daa") }
}
