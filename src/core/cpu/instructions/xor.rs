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
fn xor_u8_flags(result: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(false),
        c: Some(false),
    }
}

pub struct XorR8 {
    src: R8,
}
impl XorR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for XorR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(self.src);
        gb.cpu.a ^= n8;
        Ok(InstructionEffect::new(self.info(gb), xor_u8_flags(gb.cpu.a)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("xor {}", self.src) }
}

pub struct XorPointedByHL;
impl XorPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for XorPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        gb.cpu.a ^= n8;
        Ok(InstructionEffect::new(self.info(gb), xor_u8_flags(gb.cpu.a)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("xor [hl]") }
}

pub struct XorImm8 {
    val: u8,
}
impl XorImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for XorImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a ^= self.val;
        Ok(InstructionEffect::new(self.info(gb), xor_u8_flags(gb.cpu.a)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("xor ${:02X}", self.val) }
}
