use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
    utils::{high, low},
};

/// call given address
/// pushes the next instruction address on the stack, and then jumps to it
pub struct Call {
    pub call: IT,
}

impl Call {
    pub fn new(call: IT) -> Box<Self> { Box::new(Self { call }) }
}

impl Instruction for Call {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (addr, cycles, len) = match &self.call {
            IT::JumpToImm16(cc, addr) => {
                let should_call = cc.should_jump();

                let addr = if should_call { *addr } else { gb.cpu.pc };
                let cycles = if should_call { 6 } else { 3 };
                // TODO: return len as 0 if called?
                let len = if should_call { 0 } else { 3 };

                (addr, cycles, len)
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // push next instruction address onto the stack
        // to know where to return later
        // we're not using Push instruction to just allow to fetch Push with IT::Reg16
        // maybe this should be changed later
        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb[sp] = high(gb.cpu.sp.wrapping_add(len as u16));
        sp = sp.wrapping_sub(1);
        gb[sp] = low(gb.cpu.pc.wrapping_add(len as u16));
        gb.cpu.sp = sp;

        // implicit jump to called address
        gb.cpu.pc = addr;

        // return len as 0 if called?
        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "call {}", self.call)
    }
}
