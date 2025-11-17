use crate::core::cpu::{
    R8,
    flags::Flags,
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// Swap the upper 4 bits in register r8 and the lower 4 ones.
pub struct Swap<'a> {
    dst: ID<'a>,
}

impl<'a> Swap<'a> {
    pub fn new(dst: ID<'a>) -> Box<Self> { Box::new(Self { dst }) }
}

impl<'a> Instruction<'a> for Swap<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8, reg) if *reg != R8::F => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst = (*dst << 4) | (*dst >> 4);
        let flags = Flags {
            z: Some(*dst == 0),
            n: Some(false),
            h: Some(false),
            c: Some(false),
        };

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "swap {}", self.dst)
    }
}
