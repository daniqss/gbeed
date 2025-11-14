use std::fmt::Write;

use crate::core::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition as JC},
    },
    memory::MemoryBus,
};

/// return from subroutine and enable interrupts
pub struct Reti<'a> {
    pub pc: &'a mut u16,
    pub sp: &'a mut u16,
    pub ime: &'a mut bool,
    pub bus: MemoryBus,
    pub cc: JC,
}

impl<'a> Reti<'a> {
    pub fn new(pc: &'a mut u16, sp: &'a mut u16, ime: &'a mut bool, bus: MemoryBus, cc: JC) -> Box<Self> {
        Box::new(Self { pc, sp, ime, bus, cc })
    }
}

impl<'a> Instruction<'a> for Reti<'a> {
    fn exec(&mut self) -> InstructionResult {
        let should_return = self.cc.should_jump();
        let cycles = match &self.cc {
            JC::None => 4,
            _ if should_return => 5,
            _ => 2,
        };

        *self.ime = true;

        Ok(InstructionEffect::new(cycles, 1, Flags::none()))
    }

    // this probably is gonna look wrong
    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "reti {}", self.cc) }
}
