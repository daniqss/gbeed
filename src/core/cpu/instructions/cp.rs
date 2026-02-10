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
fn cp_flags(result: u8, a: u8, subtrahend: u8, did_borrow: bool) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(true),
        h: Some(check_borrow_hc(a, subtrahend)),
        c: Some(did_borrow),
    }
}

/// ComPare instruction
/// Compare the value in register A with the given target
/// Works by subtract the src value from register A and sets flags accordingly, but does not store the result
pub struct CpR8 {
    src: R8,
}
impl CpR8 {
    pub fn new(src: R8) -> Box<Self> { Box::new(Self { src }) }
}
impl Instruction for CpR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = gb.read(self.src);
        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            cp_flags(result, gb.cpu.a, subtrahend, did_borrow),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("cp {}", self.src) }
}

pub struct CpPointedByHL;
impl CpPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for CpPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = gb.read(gb.cpu.hl());
        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            cp_flags(result, gb.cpu.a, subtrahend, did_borrow),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("cp [hl]") }
}

pub struct CpImm8 {
    val: u8,
}
impl CpImm8 {
    pub fn new(val: u8) -> Box<Self> { Box::new(Self { val }) }
}
impl Instruction for CpImm8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let subtrahend = self.val;
        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            cp_flags(result, gb.cpu.a, subtrahend, did_borrow),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("cp ${:02X}", self.val) }
}
