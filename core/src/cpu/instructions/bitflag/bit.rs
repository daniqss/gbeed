use crate::{
    cpu::{
        R8,
        flags::{HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Test bit u3 for a 8 bit register
#[derive(Debug, Default, Clone, Copy)]
pub struct BitR8 {
    target: R8,
    bit: u8,
}

impl BitR8 {
    pub fn new(bit: u8, target: R8) -> StaticBox<Self> { StaticBox::new(Self { bit, target }) }
}
impl Instruction for BitR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.target);

        Ok(InstructionEffect::new(
            self.info(),
            Some(BitFlags::new(self.bit, r8).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("bit {}, {}", self.bit, self.target) }
}

/// Test bit u3 for the value pointed by HL
#[derive(Debug, Default, Clone, Copy)]
pub struct BitPointedByHL {
    bit: u8,
}
impl BitPointedByHL {
    pub fn new(bit: u8) -> StaticBox<Self> { StaticBox::new(Self { bit }) }
}
impl Instruction for BitPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());

        Ok(InstructionEffect::new(
            self.info(),
            Some(BitFlags::new(self.bit, n8).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (3, 2) }
    fn disassembly(&self) -> String { format!("bit {}, [hl]", self.bit) }
}

#[derive(Debug, Default, Clone, Copy)]
struct BitFlags {
    bit: u8,
    imm8: u8,
}

impl BitFlags {
    fn new(bit: u8, imm8: u8) -> StaticBox<Self> { StaticBox::new(Self { bit, imm8 }) }
}

impl LazyFlags for BitFlags {
    fn updated_flags(&self) -> u8 { ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK }

    fn zero(&self) -> bool { check_zero(self.imm8 & (1 << self.bit)) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { true }
}
