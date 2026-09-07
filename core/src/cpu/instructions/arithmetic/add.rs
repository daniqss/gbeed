use crate::{
    cpu::{
        R16,
        flags::{
            Flags, check_overflow_cy, check_overflow_cy16, check_overflow_hc, check_overflow_hc16, check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand, Operand16},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct AddA<S: Operand> {
    src: S,
}
impl<S: Operand> AddA<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for AddA<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = old_a.wrapping_add(self.src.read(gb));

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(false),
                h: Some(check_overflow_hc(gb.cpu.a, old_a)),
                c: Some(check_overflow_cy(gb.cpu.a, old_a)),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("add a,{}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddHL<S: Operand16> {
    src: S,
}
impl<S: Operand16> AddHL<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand16> Instruction for AddHL<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_hl = gb.cpu.hl();
        let result = old_hl.wrapping_add(self.src.load(gb));
        gb.store(R16::HL, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: None,
                n: Some(false),
                h: Some(check_overflow_hc16(result, old_hl)),
                c: Some(check_overflow_cy16(result, old_hl)),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("add hl,{}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddSPImm8 {
    val: i8,
}
impl AddSPImm8 {
    pub fn new(val: i8) -> Self { Self { val } }
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
    use crate::cpu::instructions::StackPointer;

    use super::*;

    #[test]
    fn test_add_hl_sp_flags() {
        let mut dmg = Dmg::default();
        let mut add_sp = AddHL::new(StackPointer);

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
