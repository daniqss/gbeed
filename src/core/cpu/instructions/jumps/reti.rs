use std::fmt::Write;

use crate::core::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    memory::MemoryBus,
};

/// return from subroutine and enable interrupts
pub struct Reti<'a> {
    pub pc: &'a mut u16,
    pub sp: &'a mut u16,
    pub ime: &'a mut bool,
    pub bus: MemoryBus,
}

impl<'a> Reti<'a> {
    pub fn new(pc: &'a mut u16, sp: &'a mut u16, ime: &'a mut bool, bus: MemoryBus) -> Box<Self> {
        Box::new(Self { pc, sp, ime, bus })
    }
}

impl<'a> Instruction<'a> for Reti<'a> {
    fn exec(&mut self) -> InstructionResult {
        *self.ime = true;

        let return_addr = self.bus.borrow().read_word(*self.sp);
        *self.pc = return_addr;
        *self.sp = self.sp.wrapping_add(2);

        // same as Ret, it actually uses 1 byte, but as it jumps, we'll leave it as 0
        Ok(InstructionEffect::new(4, 0, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "reti") }
}
