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
        match &self.call {
            IT::JumpToImm16(cc, addr) => {
                if !cc.should_jump() {
                    return Ok(InstructionEffect::new(3, 3, Flags::none()));
                }

                let return_addr = gb.cpu.pc.wrapping_add(3);

                let mut sp = gb.cpu.sp.wrapping_sub(1);
                gb[sp] = high(return_addr);

                sp = sp.wrapping_sub(1);
                gb[sp] = low(return_addr);
                gb.cpu.sp = gb.cpu.sp;

                gb.cpu.pc = *addr;

                Ok(InstructionEffect::new(6, 0, Flags::none()))
            }

            _ => Err(InstructionError::MalformedInstruction),
        }
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "call {}", self.call)
    }
}
