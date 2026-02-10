use crate::{
    Dmg,
    core::{
        cpu::{
            R8,
            flags::{Flags, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
};

#[inline(always)]
fn and_u8_flags(result: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(true),
        c: Some(false),
    }
}

pub struct AndR8 {
    src: R8,
}
impl AndR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for AndR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(self.src);
        gb.cpu.a &= n8;
        Ok(InstructionEffect::new(self.info(), and_u8_flags(gb.cpu.a)))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("and {}", self.src) }
}

pub struct AndPointedByHL;
impl AndPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for AndPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        gb.cpu.a &= n8;
        Ok(InstructionEffect::new(self.info(), and_u8_flags(gb.cpu.a)))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("and [hl]") }
}

pub struct AndImm8 {
    val: u8,
}
impl AndImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for AndImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a &= self.val;
        Ok(InstructionEffect::new(self.info(), and_u8_flags(gb.cpu.a)))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("and ${:02X}", self.val) }
}
