use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Ccf {
    carry: bool,
}

impl Ccf {
    pub fn new(carry: bool) -> StaticBox<Self> { StaticBox::new(Self { carry }) }
}

impl Instruction for Ccf {
    fn exec(&mut self, _gb: &mut Dmg) -> InstructionResult {
        Ok(InstructionEffect::new(
            self.info(),
            Some(CcfFlags::new(self.carry).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "ccf".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct CcfFlags {
    carry: bool,
}

impl CcfFlags {
    fn new(carry: bool) -> StaticBox<Self> { StaticBox::new(Self { carry }) }
}

impl LazyFlags for CcfFlags {
    fn updated_flags(&self) -> u8 { SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK }

    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { !self.carry }
}
