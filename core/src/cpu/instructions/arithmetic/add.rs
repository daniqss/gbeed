use crate::{
    cpu::{
        R8, R16,
        flags::{
            CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK,
            check_overflow_cy, check_overflow_cy16, check_overflow_hc, check_overflow_hc16, check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn add_u8(val: u8, old_a: u8) -> u8 { old_a.wrapping_add(val) }

#[inline(always)]
fn add_u16(gb: &mut Dmg, addend: u16) -> u16 {
    let result = gb.cpu.hl().wrapping_add(addend);

    gb.store(R16::HL, result);
    result
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddAR8 {
    src: R8,
}
impl AddAR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for AddAR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(gb.read(self.src), old_a);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddU8Flags::new(gb.cpu.a, old_a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("add a,{}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddAPointedByHL;
impl AddAPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for AddAPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(gb.read(gb.cpu.hl()), old_a);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddU8Flags::new(gb.cpu.a, old_a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "add a,[hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddImm8 {
    val: u8,
}
impl AddImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for AddImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(self.val, old_a);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddU8Flags::new(gb.cpu.a, old_a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("add a,${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddR16 {
    src: R16,
}

impl AddR16 {
    pub fn new(src: R16) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}

impl Instruction for AddR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_hl = gb.cpu.hl();
        let val = gb.load(self.src);
        let result = add_u16(gb, val);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddU16Flags::new(result, old_hl).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("add hl,{}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddHLSP;
impl AddHLSP {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for AddHLSP {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_hl = gb.cpu.hl();
        let val = gb.cpu.sp;
        let result = add_u16(gb, val);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddU16Flags::new(result, old_hl).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "add hl,sp".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddSPImm8 {
    val: i8,
}
impl AddSPImm8 {
    pub fn new(val: i8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for AddSPImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_sp = gb.cpu.sp;
        gb.cpu.sp = old_sp.wrapping_add(self.val as u16);

        Ok(InstructionEffect::new(
            self.info(),
            Some(AddSPImm8Flags::new(utils::low(gb.cpu.sp), utils::low(old_sp)).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("add sp,{:+}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct AddU8Flags {
    result: u8,
    old_a: u8,
}

impl AddU8Flags {
    fn new(result: u8, old_a: u8) -> StaticBox<Self> { StaticBox::new(Self { result, old_a }) }
}

impl LazyFlags for AddU8Flags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { check_overflow_hc(self.result, self.old_a) }
    fn carry(&self) -> bool { check_overflow_cy(self.result, self.old_a) }
}

#[derive(Debug, Default, Clone, Copy)]
struct AddU16Flags {
    result: u16,
    old_hl: u16,
}

impl AddU16Flags {
    fn new(result: u16, old_hl: u16) -> StaticBox<Self> { StaticBox::new(Self { result, old_hl }) }
}

impl LazyFlags for AddU16Flags {
    fn updated_flags(&self) -> u8 { SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK }

    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { check_overflow_hc16(self.result, self.old_hl) }
    fn carry(&self) -> bool { check_overflow_cy16(self.result, self.old_hl) }
}

#[derive(Debug, Default, Clone, Copy)]
struct AddSPImm8Flags {
    new_low: u8,
    old_low: u8,
}

impl AddSPImm8Flags {
    fn new(new_low: u8, old_low: u8) -> StaticBox<Self> { StaticBox::new(Self { new_low, old_low }) }
}

impl LazyFlags for AddSPImm8Flags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { false }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { check_overflow_hc(self.new_low, self.old_low) }
    fn carry(&self) -> bool { check_overflow_cy(self.new_low, self.old_low) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_hl_sp_flags() {
        let mut dmg = Dmg::default();
        let mut add_sp = AddHLSP::new();

        dmg.store(R16::HL, 0x0000);
        dmg.cpu.sp = 0x0001;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x0001);
        assert!(!flags.half_carry());
        assert!(!flags.carry());

        dmg.store(R16::HL, 0x0FFF);
        dmg.cpu.sp = 0x0001;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(flags.half_carry());
        assert!(!flags.carry());

        // carry and half Carry
        dmg.store(R16::HL, 0xFFFF);
        dmg.cpu.sp = 0x0001;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x0000);
        assert!(flags.half_carry());
        assert!(flags.carry());

        // half Carry with 0x0800 + 0x0800
        dmg.store(R16::HL, 0x0800);
        dmg.cpu.sp = 0x0800;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(flags.half_carry());
        assert!(!flags.carry());

        // carry with 0x8000 + 0x8000
        dmg.store(R16::HL, 0x8000);
        dmg.cpu.sp = 0x8000;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x0000);
        assert!(!flags.half_carry());
        assert!(flags.carry());

        // half carry edge case
        dmg.store(R16::HL, 0x0F80);
        dmg.cpu.sp = 0x0080;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(flags.half_carry());
        assert!(!flags.carry());

        // high bit addition
        dmg.store(R16::HL, 0x0001);
        dmg.cpu.sp = 0xFFFF;
        dmg.cpu.set_f(0);
        let flags = add_sp.exec(&mut dmg).unwrap().flags.unwrap();
        assert_eq!(dmg.load(R16::HL), 0x0000);
        // 0x001 + 0xFFF = 0x1000. 1+F=16. H=1.
        assert!(flags.half_carry());
        assert!(flags.carry());
    }
}
