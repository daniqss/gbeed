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
pub struct Res {
    bit: u8,
    dst: ID,
}

impl Res {
    pub fn new(bit: u8, dst: ID) -> Box<Self> { Box::new(Res { bit, dst }) }
}

impl Instruction for Res {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(reg) => (&mut gb[&*reg], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst &= !(1 << self.bit);

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "res {}, {}", self.bit, self.dst)
    }
}
