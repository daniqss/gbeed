use crate::{
    cpu::{
        flags::{
            check_overflow_cy, check_overflow_cy16, check_overflow_hc, check_overflow_hc16, check_zero, Flags,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R16, R8,
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

#[inline(always)]
fn add_u8_flags(result: u8, old_a: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(check_overflow_hc(result, old_a)),
        c: Some(check_overflow_cy(result, old_a)),
    }
}

pub struct AddAR8 {
    src: R8,
}
impl AddAR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for AddAR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(gb.read(self.src), old_a);
        Ok(InstructionEffect::new(self.info(), add_u8_flags(gb.cpu.a, old_a)))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("add a,{}", self.src) }
}

pub struct AddAPointedByHL;
impl AddAPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for AddAPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(gb.read(gb.cpu.hl()), old_a);
        Ok(InstructionEffect::new(self.info(), add_u8_flags(gb.cpu.a, old_a)))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("add a,[hl]") }
}

pub struct AddImm8 {
    val: u8,
}
impl AddImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for AddImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = add_u8(self.val, old_a);
        Ok(InstructionEffect::new(self.info(), add_u8_flags(gb.cpu.a, old_a)))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("add a,${:02X}", self.val) }
}

pub struct AddR16 {
    src: R16,
}
impl AddR16 {
    pub fn new(src: R16) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for AddR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_hl = gb.cpu.hl();
        let val = gb.load(self.src);
        let result = add_u16(gb, val);

        let flags = Flags {
            z: None,
            n: Some(false),
            h: Some(check_overflow_hc16(result, old_hl)),
            c: Some(check_overflow_cy16(result, old_hl)),
        };
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("add hl,{}", self.src) }
}

pub struct AddHLSP;
impl AddHLSP {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for AddHLSP {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_hl = gb.cpu.hl();
        let val = gb.cpu.sp;
        let result = add_u16(gb, val);

        let flags = Flags {
            z: None,
            n: Some(false),
            h: Some(check_overflow_hc16(result, old_hl)),
            c: Some(check_overflow_cy16(result, old_hl)),
        };
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("add hl,sp") }
}

pub struct AddSPImm8 {
    val: i8,
}
impl AddSPImm8 {
    pub fn new(val: i8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for AddSPImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_sp = gb.cpu.sp;
        gb.cpu.sp = old_sp.wrapping_add(self.val as u16);

        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(check_overflow_hc(utils::low(gb.cpu.sp), utils::low(old_sp))),
            c: Some(check_overflow_cy(utils::low(gb.cpu.sp), utils::low(old_sp))),
        };
        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("add sp,{:+}", self.val) }
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
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x0001);
        assert!(!dmg.cpu.half_carry());
        assert!(!dmg.cpu.carry());

        dmg.store(R16::HL, 0x0FFF);
        dmg.cpu.sp = 0x0001;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(dmg.cpu.half_carry());
        assert!(!dmg.cpu.carry());

        // carry and half Carry
        dmg.store(R16::HL, 0xFFFF);
        dmg.cpu.sp = 0x0001;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x0000);
        assert!(dmg.cpu.half_carry());
        assert!(dmg.cpu.carry());

        // half Carry with 0x0800 + 0x0800
        dmg.store(R16::HL, 0x0800);
        dmg.cpu.sp = 0x0800;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(dmg.cpu.half_carry());
        assert!(!dmg.cpu.carry());

        // carry with 0x8000 + 0x8000
        dmg.store(R16::HL, 0x8000);
        dmg.cpu.sp = 0x8000;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x0000);
        assert!(!dmg.cpu.half_carry());
        assert!(dmg.cpu.carry());

        // half carry edge case
        dmg.store(R16::HL, 0x0F80);
        dmg.cpu.sp = 0x0080;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x1000);
        assert!(dmg.cpu.half_carry());
        assert!(!dmg.cpu.carry());

        // high bit addition
        dmg.store(R16::HL, 0x0001);
        dmg.cpu.sp = 0xFFFF;
        dmg.cpu.f = 0;
        add_sp.exec(&mut dmg).unwrap().flags.apply(&mut dmg.cpu.f);
        assert_eq!(dmg.load(R16::HL), 0x0000);
        // 0x001 + 0xFFF = 0x1000. 1+F=16. H=1.
        assert!(dmg.cpu.half_carry());
        assert!(dmg.cpu.carry());
    }
}
