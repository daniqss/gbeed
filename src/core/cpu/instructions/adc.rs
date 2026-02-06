use crate::{
    Dmg,
    core::{
        cpu::{
            R8,
            flags::{Flags, check_overflow_cy, check_overflow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
};

#[inline(always)]
fn adc(addend: u8, old_a: u8, carry: bool) -> u8 {
    old_a.wrapping_add(addend).wrapping_add(if carry { 1 } else { 0 })
}

#[inline(always)]
fn adc_flags(result: u8, old_a: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(check_overflow_hc(result, old_a)),
        c: Some(check_overflow_cy(result, old_a)),
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
        let old_a = gb.cpu.a;
        gb.cpu.a = adc(gb.read(self.src), old_a, gb.cpu.carry());
        Ok(InstructionEffect::new(self.info(), adc_flags(gb.cpu.a, old_a)))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("adc a,{}", self.src) }
}

/// Add with carry instruction
/// Adds the value pointed by HL plus the carry flag to register A
pub struct AdcPointedByHL {
    carry: bool,
}
impl AdcPointedByHL {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}
impl Instruction for AdcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = adc(gb.read(gb.cpu.hl()), old_a, gb.cpu.carry());
        Ok(InstructionEffect::new(self.info(), adc_flags(gb.cpu.a, old_a)))
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
        let old_a = gb.cpu.a;
        gb.cpu.a = adc(self.val, old_a, gb.cpu.carry());
        Ok(InstructionEffect::new(self.info(), adc_flags(gb.cpu.a, old_a)))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("adc a,${:02X}", self.val) }
}
