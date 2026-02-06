use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
    },
};

/// Sets bit u3 in register r8 to 0. Bit 0 is the rightmost one, bit 7 the leftmost one
pub struct Set {
    bit: u8,
    dst: ID,
}

impl Set {
    pub fn new(bit: u8, dst: ID) -> Box<Self> { Box::new(Set { bit, dst }) }
}

impl Instruction for Set {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(reg) => (&mut gb[&*reg], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst |= 1 << self.bit;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("set {}, {}", self.bit, self.dst) }
}
