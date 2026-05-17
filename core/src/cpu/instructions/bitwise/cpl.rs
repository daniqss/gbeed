use crate::{
    cpu::{
        flags::{HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Bitwise NOT, ComPLement accumulator.
/// Only operates on register A
#[derive(Debug, Default, Clone, Copy)]
pub struct Cpl;

impl Cpl {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Cpl {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.a = !gb.cpu.a;

        Ok(InstructionEffect::new(self.info(), Some(CplFlags::new().into())))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "cpl".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct CplFlags;

impl CplFlags {
    fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl LazyFlags for CplFlags {
    fn updated_flags(&self) -> u8 { SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK }

    fn subtraction(&self) -> bool { true }
    fn half_carry(&self) -> bool { true }
}
