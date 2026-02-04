use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::Flags,
            instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition as JC},
        },
        memory::{Accessible, Accessible16},
    },
};

/// return from subroutine
pub struct Ret {
    pub cc: JC,
}

impl Ret {
    pub fn new(cc: JC) -> Box<Self> { Box::new(Self { cc }) }
}

impl Instruction for Ret {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let should_return = self.cc.should_jump();
        let cycles = match &self.cc {
            JC::None => 4,
            _ if should_return => 5,
            _ => 2,
        };

        if !should_return {
            return Ok(InstructionEffect::new(cycles, 1, Flags::none()));
        }

        let return_addr = gb.load(gb.cpu.sp);
        gb.cpu.pc = return_addr;
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::with_jump(cycles, 1, Flags::none()))
    }

    // this probably is gonna look wrong
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "ret {}", self.cc) }
}
