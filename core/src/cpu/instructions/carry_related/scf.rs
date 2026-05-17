use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Scf;

impl Scf {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Scf {
    fn exec(&mut self, _gb: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(self.info(), Some(ScfFlags::new().into())))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "scf".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct ScfFlags;

impl ScfFlags {
    fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl LazyFlags for ScfFlags {
    fn updated_flags(&self) -> u8 { SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK }

    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { true }
}
