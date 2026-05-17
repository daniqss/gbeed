use crate::{
    cpu::{
        R8,
        flags::{
            CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK,
            check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Add with carry instruction
/// Adds the value of the wanted register plus the carry flag to register A
#[derive(Debug, Default, Clone, Copy)]
pub struct AdcR8 {
    src: R8,
}
impl AdcR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for AdcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.src);

        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_add(val).wrapping_add(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(AdcFlags::new(result, old_a, val, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("adc a,{}", self.src) }
}

/// Add with carry instruction
/// Adds the value pointed by HL plus the carry flag to register A
#[derive(Debug, Default, Clone, Copy)]
pub struct AdcPointedByHL;
impl AdcPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for AdcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());

        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_add(val).wrapping_add(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(AdcFlags::new(result, old_a, val, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "adc a,[hl]".to_string() }
}

/// Add with carry instruction
/// Adds the value of the immediate 8 bit value plus the carry flag to register A
#[derive(Debug, Default, Clone, Copy)]
pub struct AdcImm8 {
    val: u8,
}
impl AdcImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for AdcImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_add(self.val).wrapping_add(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(AdcFlags::new(result, old_a, self.val, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("adc a,${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct AdcFlags {
    result: u8,
    old_a: u8,
    val: u8,
    carry_in: u8,
}

impl AdcFlags {
    fn new(result: u8, old_a: u8, val: u8, carry_in: u8) -> StaticBox<Self> {
        StaticBox::new(Self {
            result,
            old_a,
            val,
            carry_in,
        })
    }
}

impl LazyFlags for AdcFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { (self.old_a & 0xF) + (self.val & 0xF) + self.carry_in > 0xF }
    fn carry(&self) -> bool { (self.old_a as u16) + (self.val as u16) + (self.carry_in as u16) > 0xFF }
}
