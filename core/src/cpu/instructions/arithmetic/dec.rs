use crate::{
    cpu::{
        R8, R16,
        flags::{
            HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK, check_borrow_hc,
            check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    memory::Accessible,
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct DecR8 {
    dst: R8,
}
impl DecR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for DecR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_sub(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(DecU8Flags::new(r8, result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecPointedByHL;
impl DecPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for DecPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_sub(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(DecU8Flags::new(n8, result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { "dec [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecR16 {
    dst: R16,
}
impl DecR16 {
    pub fn new(dst: R16) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for DecR16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_sub(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DecStackPointer;

impl DecStackPointer {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for DecStackPointer {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(1);

        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (2, 1) }
    fn disassembly(&self) -> String { "dec sp".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct DecU8Flags {
    old: u8,
    result: u8,
}

impl DecU8Flags {
    fn new(old: u8, result: u8) -> StaticBox<Self> { StaticBox::new(Self { old, result }) }
}

impl LazyFlags for DecU8Flags {
    fn updated_flags(&self) -> u8 { ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { true }
    fn half_carry(&self) -> bool { check_borrow_hc(self.old, 1) }
}
