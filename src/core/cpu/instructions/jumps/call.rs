use std::fmt::Write;

use crate::{
    core::{
        cpu::{
            flags::Flags,
            instructions::{
                Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
            },
        },
        memory::MemoryBus,
    },
    utils::{high, low},
};

/// call given address
/// pushes the next instruction address on the stack, and then jumps to it
pub struct Call<'a> {
    pub pc: &'a mut u16,
    pub sp: &'a mut u16,
    pub bus: MemoryBus,
    pub call: IT<'a>,
}

impl<'a> Call<'a> {
    pub fn new(pc: &'a mut u16, sp: &'a mut u16, bus: MemoryBus, call: IT<'a>) -> Box<Self> {
        Box::new(Self { pc, sp, bus, call })
    }
}

impl<'a> Instruction<'a> for Call<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (addr, cycles, len) = match &self.call {
            IT::JumpToImm16(cc, addr) => {
                let should_call = cc.should_jump();

                let addr = if should_call { *addr } else { *self.pc };
                let cycles = if should_call { 6 } else { 3 };

                (addr, cycles, 3)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // push next instruction address onto the stack
        // to know where to return later
        // we're not using Push instruction to just allow to fetch Push with IT::Reg16
        // maybe this should be changed later
        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = high(self.pc.wrapping_add(len as u16));
        *self.sp -= 1;
        self.bus.borrow_mut()[*self.sp] = low(self.pc.wrapping_add(len as u16));

        // implicit jump to called address
        *self.pc = addr;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "call {}", self.call)
    }
}
