use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

/// jump to the given address
/// it can get a condition to jump only if the condition is met
/// this condition is based on carry and zero flags
pub struct Jp {
    pub jump: IT,
}

impl Jp {
    pub fn new(jump: IT) -> Box<Self> { Box::new(Self { jump }) }
}

impl Instruction for Jp {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (addr, cycles, len) = match &self.jump {
            IT::JumpToImm16(cc, addr) => {
                let should_jump = cc.should_jump();

                let addr = if should_jump { *addr } else { gb.cpu.pc };
                let cycles = if should_jump { 4 } else { 3 };
                // TODO: return len as 0 if jumped?
                let len = if should_jump { 0 } else { 3 };

                (addr, cycles, len)
            }

            IT::JumpToHL(addr) => (*addr, 1, 0),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        gb.cpu.pc = addr;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "jp {}", self.jump) }
}
