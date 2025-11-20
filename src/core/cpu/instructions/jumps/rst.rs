use std::fmt::Write;

use crate::{
    core::{
        cpu::{
            flags::Flags,
            instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
        },
        memory::MemoryBus,
    },
    utils::{high, low},
};

/// restart instruction, same that call, but faster for suitable addresses
/// used to call a interruption routine
pub struct Rst<'a> {
    pub pc: &'a mut u16,
    pub sp: &'a mut u16,
    pub bus: MemoryBus,
    pub vec: u8,
}

impl<'a> Rst<'a> {
    pub fn new(pc: &'a mut u16, sp: &'a mut u16, bus: MemoryBus, vec: u8) -> Box<Self> {
        Box::new(Self { pc, sp, bus, vec })
    }
}

impl<'a> Instruction<'a> for Rst<'a> {
    fn exec(&mut self) -> InstructionResult {
        [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38]
            .iter()
            .position(|&x| x == self.vec)
            .ok_or(InstructionError::MalformedInstruction)?;

        // maybe this logic should be shared with call
        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = high(self.pc.wrapping_add(1));
        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = low(self.pc.wrapping_add(1));

        // implicit jump to called address
        *self.pc = self.vec as u16;

        // TODO: return 0 instead of 1 len to avoid pc increment after instruction?
        Ok(InstructionEffect::new(4, 0, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "rst ${:02X}", self.vec)
    }
}
