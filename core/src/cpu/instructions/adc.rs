use crate::{
    cpu::{
        flags::{check_zero, Flags},
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R8,
    },
    prelude::*,
};

#[inline(always)]
fn adc(gb: &mut Dmg, val: u8) -> Flags {
    let old_a = gb.cpu.a;
    let carry = if gb.cpu.carry() { 1 } else { 0 };

    let result = old_a.wrapping_add(val).wrapping_add(carry);
    gb.cpu.a = result;

    let h_check = (old_a & 0xF) + (val & 0xF) + carry > 0xF;
    let c_check = (old_a as u16) + (val as u16) + (carry as u16) > 0xFF;

    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(h_check),
        c: Some(c_check),
    }
}

/// Add with carry instruction
/// Adds the value of the wanted register plus the carry flag to register A
pub struct AdcR8 {
    src: R8,
}
impl AdcR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for AdcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.src);
        let flags = adc(gb, val);
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("adc a,{}", self.src) }
}

/// Add with carry instruction
/// Adds the value pointed by HL plus the carry flag to register A
pub struct AdcPointedByHL;
impl AdcPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for AdcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let flags = adc(gb, val);
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("adc a,[hl]") }
}

/// Add with carry instruction
/// Adds the value of the immediate 8 bit value plus the carry flag to register A
pub struct AdcImm8 {
    val: u8,
}
impl AdcImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for AdcImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let flags = adc(gb, self.val);
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("adc a,${:02X}", self.val) }
}
