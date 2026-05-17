use crate::{
    cpu::{
        R8,
        flags::{ALL_FLAGS_MASK, LazyFlags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct XorR8 {
    src: R8,
}
impl XorR8 {
    pub fn new(src: R8) -> StaticBox<Self> { StaticBox::new(Self { src }) }
}
impl Instruction for XorR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(self.src);
        gb.cpu.a ^= n8;
        Ok(InstructionEffect::new(
            self.info(),
            Some(XorFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("xor {}", self.src) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XorPointedByHL;
impl XorPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for XorPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        gb.cpu.a ^= n8;
        Ok(InstructionEffect::new(
            self.info(),
            Some(XorFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "xor [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct XorImm8 {
    val: u8,
}
impl XorImm8 {
    pub fn new(val: u8) -> StaticBox<Self> { StaticBox::new(Self { val }) }
}
impl Instruction for XorImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a ^= self.val;
        Ok(InstructionEffect::new(
            self.info(),
            Some(XorFlags::new(gb.cpu.a).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("xor ${:02X}", self.val) }
}

#[derive(Debug, Default, Clone, Copy)]
struct XorFlags {
    result: u8,
}

impl XorFlags {
    fn new(result: u8) -> StaticBox<Self> { StaticBox::new(Self { result }) }
}

impl LazyFlags for XorFlags {
    fn updated_flags(&self) -> u8 { ALL_FLAGS_MASK }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { false }
}
