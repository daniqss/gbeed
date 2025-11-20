use std::fmt::Write;

use crate::core::cpu::{
    flags::Flags,
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
    },
};

/// relative jump to the given address
/// it can get a condition to jump only if the condition is met
/// the condition is based on carry and zero flags
/// the address is encoded as a signed 8 bit immediate value
pub struct Jr<'a> {
    pub pc: &'a mut u16,
    pub jump: IT<'a>,
}

impl<'a> Jr<'a> {
    pub fn new(pc: &'a mut u16, jump: IT<'a>) -> Box<Self> { Box::new(Self { pc, jump }) }
}

impl<'a> Instruction<'a> for Jr<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (offset, cycles, len) = match &self.jump {
            IT::JumpToImm8(cc, offset) => {
                let should_jump = cc.should_jump();

                // cast i8 offset to u16 to perform addition
                let offset = if should_jump { *offset as i16 as u16 } else { 0 };
                let cycles = if should_jump { 3 } else { 2 };
                // TODO: return len as 0 if jumped?
                let len = if should_jump { 0 } else { 2 };

                (offset, cycles, len)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *self.pc = self.pc.wrapping_add(offset);

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "jr {}", self.jump) }
}
