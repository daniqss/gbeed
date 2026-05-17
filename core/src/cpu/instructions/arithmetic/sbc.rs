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

/// Subtraction with carry instruction
/// Subtracts the value of the wanted register from register A, and the carry flag
#[derive(Debug, Default, Clone, Copy)]
pub struct SbcR8 {
    src: R8,
}
impl SbcR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for SbcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.src);

        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_sub(val).wrapping_sub(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(SbcFlags::new(result, old_a, val, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("sbc a,{}", self.src) }
}

/// Subtraction with carry instruction
/// Subtracts the value pointed by HL from register A, and the carry flag
#[derive(Debug, Default, Clone, Copy)]
pub struct SbcPointedByHL;
impl SbcPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SbcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());

        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_sub(val).wrapping_sub(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(SbcFlags::new(result, old_a, val, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "sbc a,[hl]".to_string() }
}

/// Subtraction with carry instruction
/// Subtracts the value of the immediate 8 bit value from register A, and the carry flag
#[derive(Debug, Default, Clone, Copy)]
pub struct SbcImm8 {
    n8: u8,
}
impl SbcImm8 {
    pub fn new(n8: u8) -> StaticBox<Self> { StaticBox::new(Self { n8 }) }
}
impl Instruction for SbcImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let carry_in: u8 = if gb.cpu.carry() { 1 } else { 0 };

        let result = old_a.wrapping_sub(self.n8).wrapping_sub(carry_in);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(
            self.info(),
            Some(SbcFlags::new(result, old_a, self.n8, carry_in).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sbc a,${:02X}", self.n8) }
}

#[derive(Debug, Default, Clone, Copy)]
struct SbcFlags {
    result: u8,
    old_a: u8,
    val: u8,
    carry_in: u8,
}

impl SbcFlags {
    fn new(result: u8, old_a: u8, val: u8, carry_in: u8) -> StaticBox<Self> {
        StaticBox::new(Self {
            result,
            old_a,
            val,
            carry_in,
        })
    }
}

impl LazyFlags for SbcFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { true }
    fn half_carry(&self) -> bool { (self.old_a & 0xF) < (self.val & 0xF) + self.carry_in }
    fn carry(&self) -> bool { (self.old_a as u16) < (self.val as u16) + (self.carry_in as u16) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbc_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        gb.cpu.set_carry();
        let mut instr = SbcImm8::new(19);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(flags.zero());
        assert!(flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }

    #[test]
    fn test_sbc_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        gb.cpu.b = 0b0000_0011;
        gb.cpu.clear_carry();

        let mut instr = SbcR8::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0b0000_1101);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(flags.subtraction());
        assert!(flags.half_carry());
        assert!(!flags.carry());
    }

    #[test]
    fn test_sbc_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        gb.write(0xC020, 0x20);
        gb.cpu.h = 0xC0;
        gb.cpu.l = 0x20;
        gb.cpu.clear_carry();

        let mut instr = SbcPointedByHL::new();
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_sbc_with_carry_flag() {
        let mut gb = Dmg::default();
        gb.cpu.a = 10;
        gb.cpu.b = 3;
        gb.cpu.set_carry();

        let mut instr = SbcR8::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 6);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());

        gb.cpu.a = 5;
        gb.cpu.set_carry();
        let mut instr = SbcImm8::new(5);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 255);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(flags.subtraction());
        assert!(flags.half_carry());
        assert!(flags.carry());
    }
}
