use std::fmt::Write;

use crate::core::cpu::{
    flags::Flags,
    instructions::{
        Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        JumpCondition as JC,
    },
};

/// jump to the given address
/// it can get a condition to jump only if the condition is met
/// this condition is based on the zero flag
pub struct Jp<'a> {
    pub pc: &'a mut u16,
    pub jump: IT<'a>,
}

impl<'a> Jp<'a> {
    pub fn new(pc: &'a mut u16, jump: IT<'a>) -> Box<Self> { Box::new(Self { pc, jump }) }
}

impl<'a> Instruction<'a> for Jp<'a> {
    fn exec(&mut self) -> InstructionResult {
        // let (addr, cycles, len) = match &self.jump {
        //     IT::JumpToImm16(JC::None, addr) => (*addr, 4, 3),
        //     IT::JumpToImm16(JC::Zero(true), addr) => (*addr, 4, 3),
        //     IT::JumpToImm16(JC::Zero(false), _) => (*self.pc, 3, 3),
        //     IT::JumpToImm16(JC::NotZero(true), addr) => (*addr, 4, 3),
        //     IT::JumpToImm16(JC::NotZero(false), _) => (*self.pc, 3, 3),
        //     IT::JumpToImm16(JC::Carry(true), addr) => (*addr, 4, 3),
        //     IT::JumpToImm16(JC::Carry(false), _) => (*self.pc, 3, 3),
        //     IT::JumpToImm16(JC::NotCarry(true), addr) => (*addr, 4, 3),
        //     IT::JumpToImm16(JC::NotCarry(false), _) => (*self.pc, 3, 3),
        //     IT::JumpToHL(addr) => (*addr, 1, 1),

        //     _ => return Err(InstructionError::MalformedInstruction),
        // };
        let (addr, cycles, len) = match &self.jump {
            IT::JumpToImm16(cc, addr) => {
                let should_jump = cc.should_jump();

                let addr = if should_jump { *addr } else { *self.pc };
                let cycles = if should_jump { 4 } else { 3 };

                (addr, cycles, 3)
            }

            IT::JumpToHL(addr) => (*addr, 1, 1),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *self.pc = addr;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "jp {}", self.jump) }
}
