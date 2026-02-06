use crate::{
    Dmg,
    core::{
        Accessible,
        cpu::{
            R8,
            flags::{Flags, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
    },
};

#[inline(always)]
fn bit_flags(test_bit: u8) -> Flags {
    Flags {
        z: Some(check_zero(test_bit)),
        n: Some(false),
        h: Some(true),
        c: None,
    }
}

#[inline(always)]
fn test_bit(value: u8, bit: u8) -> u8 { value & (1 << bit) }

// /// Test bit u3 for a 8 bit register
pub struct BitR8 {
    target: R8,
    bit: u8,
}

impl BitR8 {
    pub fn new(bit: u8, target: R8) -> Box<Self> { Box::new(Self { bit, target }) }
}
impl Instruction for BitR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.target);
        let test_bit = test_bit(r8, self.bit);

        Ok(InstructionEffect::new(self.info(), bit_flags(test_bit)))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("bit {}, {}", self.bit, self.target) }
}

/// Test bit u3 for the value pointed by HL
pub struct BitPointedByHL {
    bit: u8,
}
impl BitPointedByHL {
    pub fn new(bit: u8) -> Box<Self> { Box::new(Self { bit }) }
}
impl Instruction for BitPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let test_bit = test_bit(n8, self.bit);

        Ok(InstructionEffect::new(self.info(), bit_flags(test_bit)))
    }
    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String { format!("bit {}, [hl]", self.bit) }
}
