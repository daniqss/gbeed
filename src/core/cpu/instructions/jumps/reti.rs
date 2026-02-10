use crate::{
    Dmg,
    core::{
        cpu::{
            flags::Flags,
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible16,
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

        let return_addr = gb.load(gb.cpu.sp);
        gb.cpu.pc = return_addr;
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (4, 1) }
    fn disassembly(&self) -> String { format!("reti") }
}
