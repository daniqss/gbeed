use crate::{
    cpu::{
        R8,
        flags::{
            CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK,
            check_borrow_hc, check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn sub(subtrahend: u8, old_a: u8) -> u8 { old_a.wrapping_sub(subtrahend) }

/// Subtraction instruction
/// Subtracts the value of the wanted register from register A
#[derive(Debug, Default, Clone, Copy)]
pub struct SubR8 {
    src: R8,
}
impl SubR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for SubR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(self.src);
        gb.cpu.a = sub(subtrahend, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            Some(SubFlags::new(gb.cpu.a, old_a, subtrahend).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("sub a,{}", self.src) }
}

/// Subtraction instruction
/// Subtracts the value pointed by HL from register A
#[derive(Debug, Default, Clone, Copy)]
pub struct SubPointedByHL;
impl SubPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SubPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(gb.cpu.hl());
        gb.cpu.a = sub(subtrahend, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            Some(SubFlags::new(gb.cpu.a, old_a, subtrahend).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "sub a,[hl]".to_string() }
}

/// Subtraction instruction
/// Subtracts the value of the immediate 8 bit value from register A
#[derive(Debug, Default, Clone, Copy)]
pub struct SubImm8 {
    val: u8,
}
impl SubImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for SubImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = sub(self.val, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            Some(SubFlags::new(gb.cpu.a, old_a, self.val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sub a,${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct SubFlags {
    result: u8,
    old_a: u8,
    subtrahend: u8,
}

impl SubFlags {
    fn new(result: u8, old_a: u8, subtrahend: u8) -> StaticBox<Self> {
        StaticBox::new(Self {
            result,
            old_a,
            subtrahend,
        })
    }
}

impl LazyFlags for SubFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { true }
    fn half_carry(&self) -> bool { check_borrow_hc(self.old_a, self.subtrahend) }
    fn carry(&self) -> bool { self.old_a < self.subtrahend }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        let mut instr = SubImm8::new(20);

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
    fn test_sub_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        gb.cpu.b = 0b0000_0001;

        let mut instr = SubR8::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0x0F);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(flags.subtraction());
        assert!(flags.half_carry());
        assert!(!flags.carry());
    }

    #[test]
    fn test_sub_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        gb.write(0xC020, 0x20);
        gb.cpu.h = 0xC0;
        gb.cpu.l = 0x20;

        let mut instr = SubPointedByHL::new();
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
}
