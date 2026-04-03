use crate::{
    cpu::{
        R8, R16,
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    memory::Accessible,
    prelude::*,
};

#[inline(always)]
fn dec_u8_flags(old: u8, result: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(true),
        h: Some(check_borrow_hc(old, 1)),
        c: None,
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecR8 {
    dst: R8,
}
impl DecR8 {
    pub fn new(dst: R8) -> InstructionBox { InstructionBox::new(Self { dst }) }
}

impl Instruction for DecR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_sub(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), dec_u8_flags(r8, result)))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecPointedByHL;
impl DecPointedByHL {
    pub fn new() -> InstructionBox { InstructionBox::new(Self) }
}

impl Instruction for DecPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_sub(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), dec_u8_flags(n8, result)))
    }
    fn info(&self) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { "dec [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecR16 {
    dst: R16,
}
impl DecR16 {
    pub fn new(dst: R16) -> InstructionBox { InstructionBox::new(Self { dst }) }
}

impl Instruction for DecR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_sub(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecStackPointer;

impl DecStackPointer {
    pub fn new() -> InstructionBox { InstructionBox::new(Self) }
}

impl Instruction for DecStackPointer {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(1);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "dec sp".to_string() }
}
