use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
    },
    utils::{high, low},
};

/// restart instruction, same that call, but faster for suitable addresses
/// used to call a interruption routine
pub struct Rst {
    pub vec: u8,
}

impl Rst {
    pub fn new(vec: u8) -> Box<Self> { Box::new(Self { vec }) }
}

impl Instruction for Rst {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        // [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38]
        //     .iter()
        //     .position(|&x| x == self.vec)
        //     .ok_or(InstructionError::MalformedInstruction)?;
        if self.vec & 0x07 != 0 {
            return Err(InstructionError::MalformedInstruction);
        }

        // maybe this logic should be shared with call
        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb[sp] = high(gb.cpu.sp.wrapping_add(1));
        sp = sp.wrapping_sub(1);
        gb[sp] = low(gb.cpu.pc.wrapping_add(1));
        gb.cpu.sp = sp;

        // implicit jump to called address
        gb.cpu.pc = self.vec as u16;

        // TODO: return 0 instead of 1 len to avoid pc increment after instruction?
        Ok(InstructionEffect::new(4, 0, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "rst ${:02X}", self.vec)
    }
}
