use crate::{
    cpu::{
        R8,
        flags::{ALL_FLAGS_MASK, LazyFlags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct AndR8 {
    src: R8,
}
impl AndR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for AndR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(self.src);
        gb.cpu.a &= n8;

        Ok(InstructionEffect::new(
            self.info(),
            Some(AndFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("and {}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AndPointedByHL;
impl AndPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for AndPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        gb.cpu.a &= n8;
        Ok(InstructionEffect::new(
            self.info(),
            Some(AndFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "and [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AndImm8 {
    val: u8,
}
impl AndImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for AndImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a &= self.val;
        Ok(InstructionEffect::new(
            self.info(),
            Some(AndFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("and ${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct AndFlags {
    result: u8,
}

impl AndFlags {
    fn new(result: u8) -> StaticBox<Self> { StaticBox::new(Self { result }) }
}

impl LazyFlags for AndFlags {
    fn updated_flags(&self) -> u8 { ALL_FLAGS_MASK }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { true }
    fn carry(&self) -> bool { false }
}
