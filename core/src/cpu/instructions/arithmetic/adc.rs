use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

/// Add with carry instruction
/// Adds the given operand plus the carry flag to register A
#[derive(Debug, Default, Clone, Copy)]
pub struct Adc<S: Operand> {
    src: S,
}
impl<S: Operand> Adc<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for Adc<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let val = self.src.read(gb);
        let carry = if gb.cpu.carry() { 1 } else { 0 };
        gb.cpu.a = old_a.wrapping_add(val).wrapping_add(carry);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(false),
                h: Some((old_a & 0xF) + (val & 0xF) + carry > 0xF),
                c: Some((old_a as u16) + (val as u16) + (carry as u16) > 0xFF),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("adc a,{}", self.src) }
}
