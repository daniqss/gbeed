use crate::{
    cpu::{
        flags::{check_borrow_hc, check_zero, Flags},
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R8,
    },
    prelude::*,
};

#[inline(always)]
fn sub(subtrahend: u8, old_a: u8) -> u8 { old_a.wrapping_sub(subtrahend) }

#[inline(always)]
fn sub_flags(result: u8, old_a: u8, subtrahend: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(true),
        h: Some(check_borrow_hc(old_a, subtrahend)),
        c: Some(old_a < subtrahend),
    }
}

/// Subtraction instruction
/// Subtracts the value of the wanted register from register A
pub struct SubR8 {
    src: R8,
}
impl SubR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for SubR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(self.src);
        gb.cpu.a = sub(subtrahend, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            sub_flags(gb.cpu.a, old_a, subtrahend),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("sub a,{}", self.src) }
}

/// Subtraction instruction
/// Subtracts the value pointed by HL from register A
pub struct SubPointedByHL;
impl SubPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}
impl Instruction for SubPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = gb.read(gb.cpu.hl());
        gb.cpu.a = sub(subtrahend, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            sub_flags(gb.cpu.a, old_a, subtrahend),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("sub a,[hl]") }
}

/// Subtraction instruction
/// Subtracts the value of the immediate 8 bit value from register A
pub struct SubImm8 {
    val: u8,
}
impl SubImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for SubImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        gb.cpu.a = sub(self.val, old_a);
        Ok(InstructionEffect::new(
            self.info(),
            sub_flags(gb.cpu.a, old_a, self.val),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sub a,${:02X}", self.val) }
}

#[cfg(test)]
mod tests {
    use crate::cpu::flags::Flags;

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
    fn test_sub_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        gb.cpu.b = 0b0000_0001;

        let mut instr = SubR8::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0x0F);
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
