use crate::{
    cpu::{
        R8,
        flags::{ALL_FLAGS_MASK, LazyFlags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct OrR8 {
    src: R8,
}
impl OrR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for OrR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(self.src);
        gb.cpu.a |= n8;
        Ok(InstructionEffect::new(
            self.info(),
            Some(OrFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("or {}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrPointedByHL;
impl OrPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for OrPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        gb.cpu.a |= n8;
        Ok(InstructionEffect::new(
            self.info(),
            Some(OrFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "or [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrImm8 {
    val: u8,
}
impl OrImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for OrImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a |= self.val;
        Ok(InstructionEffect::new(
            self.info(),
            Some(OrFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("or ${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct OrFlags {
    result: u8,
}

impl OrFlags {
    fn new(result: u8) -> StaticBox<Self> { StaticBox::new(Self { result }) }
}

impl LazyFlags for OrFlags {
    fn updated_flags(&self) -> u8 { ALL_FLAGS_MASK }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { false }
}
