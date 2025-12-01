use std::fmt::Write;

use crate::{
    Dmg,
    core::{
        cpu::{
            flags::Flags,
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessable,
    },
};

/// return from subroutine and enable interrupts
pub struct Reti {}

impl Reti {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Reti {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.ime = true;

        let return_addr = gb.read16(gb.cpu.sp);
        gb.cpu.pc = return_addr;
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::with_jump(4, 1, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "reti") }
}
