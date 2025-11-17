use std::fmt::Write;

use crate::core::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition as JC},
    },
    memory::MemoryBus,
};

/// return from subroutine
pub struct Ret<'a> {
    pub pc: &'a mut u16,
    pub sp: &'a mut u16,
    pub bus: MemoryBus,
    pub cc: JC,
}

impl<'a> Ret<'a> {
    pub fn new(pc: &'a mut u16, sp: &'a mut u16, bus: MemoryBus, cc: JC) -> Box<Self> {
        Box::new(Self { pc, sp, bus, cc })
    }
}

impl<'a> Instruction<'a> for Ret<'a> {
    fn exec(&mut self) -> InstructionResult {
        let should_return = self.cc.should_jump();
        let cycles = match &self.cc {
            JC::None => 4,
            _ if should_return => 5,
            _ => 2,
        };

        if !should_return {
            return Ok(InstructionEffect::new(cycles, 1, Flags::none()));
        }

        let return_addr = self.bus.borrow().read_word(*self.sp);
        *self.pc = return_addr;
        *self.sp = self.sp.wrapping_add(2);

        // it actually uses 1 byte, but as it jumps, we'll leave it as 0
        Ok(InstructionEffect::new(cycles, 0, Flags::none()))
    }

    // this probably is gonna look wrong
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "ret {}", self.cc) }
}
