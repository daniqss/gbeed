use crate::{
    Dmg,
    core::{
        Accessible,
        cpu::{
            R8,
            flags::{Flags, check_borrow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
    },
};

#[inline(always)]
fn sbc(subtrahend: u8, old_a: u8, carry: bool) -> (u8, bool, bool) {
    let (result, did_borrow_sub) = old_a.overflowing_sub(subtrahend);
    let (result, did_borrow_cy) = result.overflowing_sub(if carry { 1 } else { 0 });
    (result, did_borrow_sub, did_borrow_cy)
}

#[inline(always)]
fn sbc_flags(result: u8, old_a: u8, subtrahend: u8, did_borrow_sub: bool, did_borrow_cy: bool) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(true),
        h: Some(check_borrow_hc(old_a, subtrahend)),
        c: Some(did_borrow_sub || did_borrow_cy),
    }
}

/// Subtraction with carry instruction
/// Subtracts the value of the wanted register from register A, and the carry flag
pub struct SbcR8 {
    src: R8,
}
impl SbcR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for SbcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(self.src);
        let (result, did_borrow_sub, did_borrow_cy) = sbc(subtrahend, old_a, gb.cpu.carry());
        gb.cpu.a = result;
        Ok(InstructionEffect::new(
            self.info(),
            sbc_flags(gb.cpu.a, old_a, subtrahend, did_borrow_sub, did_borrow_cy),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("sbc a,{}", self.src) }
}

/// Subtraction with carry instruction
/// Subtracts the value pointed by HL from register A, and the carry flag
pub struct SbcPointedByHL;
impl SbcPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for SbcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(gb.cpu.hl());
        let (result, did_borrow_sub, did_borrow_cy) = sbc(subtrahend, old_a, gb.cpu.carry());
        gb.cpu.a = result;
        Ok(InstructionEffect::new(
            self.info(),
            sbc_flags(gb.cpu.a, old_a, subtrahend, did_borrow_sub, did_borrow_cy),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("sbc a,[hl]") }
}

/// Subtraction with carry instruction
/// Subtracts the value of the immediate 8 bit value from register A, and the carry flag
pub struct SbcImm8 {
    n8: u8,
    carry: bool,
}
impl SbcImm8 {
    pub fn new(n8: u8, carry: bool) -> Box<Self> { Box::new(Self { n8, carry }) }
}
impl Instruction for SbcImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let (result, did_borrow_sub, did_borrow_cy) = sbc(self.n8, old_a, self.carry);
        gb.cpu.a = result;
        Ok(InstructionEffect::new(
            self.info(),
            sbc_flags(gb.cpu.a, old_a, self.n8, did_borrow_sub, did_borrow_cy),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sbc a,${:02X}", self.n8) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbc_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        gb.cpu.set_carry();
        let mut instr = SbcImm8::new(19, gb.cpu.carry());

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(true),
                h: Some(false),
                c: Some(false),
            }
        );
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
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(true),
                c: Some(false),
            }
        );
    }

    #[test]
    fn test_sbc_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        gb.write(0x20, 0x20);
        gb.cpu.h = 0x00;
        gb.cpu.l = 0x20;
        gb.write(R8::F, 0);

        let mut instr = SbcPointedByHL::new();
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(true),
            }
        );
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
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(false),
            }
        );

        gb.cpu.a = 5;
        gb.cpu.set_carry();
        let mut instr = SbcImm8::new(5, gb.cpu.carry());
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 255);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(true),
            }
        );
    }
}
