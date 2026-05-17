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

/// ComPare instruction
/// Compare the value in register A with the given target
/// Works by subtract the src value from register A and sets flags accordingly, but does not store the result
#[derive(Debug, Default, Clone, Copy)]
pub struct CpR8 {
    src: R8,
}
impl CpR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for CpR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = gb.read(self.src);
        let a = gb.cpu.a;
        let result = a.wrapping_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            Some(CpFlags::new(result, a, subtrahend).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("cp {}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CpPointedByHL;
impl CpPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for CpPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = gb.read(gb.cpu.hl());
        let a = gb.cpu.a;
        let result = a.wrapping_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            Some(CpFlags::new(result, a, subtrahend).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "cp [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CpImm8 {
    val: u8,
}
impl CpImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for CpImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = self.val;
        let a = gb.cpu.a;
        let result = a.wrapping_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            Some(CpFlags::new(result, a, subtrahend).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("cp ${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct CpFlags {
    result: u8,
    a: u8,
    subtrahend: u8,
}

impl CpFlags {
    fn new(result: u8, a: u8, subtrahend: u8) -> StaticBox<Self> {
        StaticBox::new(Self {
            result,
            a,
            subtrahend,
        })
    }
}

impl LazyFlags for CpFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { true }
    fn half_carry(&self) -> bool { check_borrow_hc(self.a, self.subtrahend) }
    fn carry(&self) -> bool { self.a < self.subtrahend }
}
