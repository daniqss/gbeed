use crate::core::cpu::{
    flags::Flags,
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// Sets bit u3 in register r8 to 0. Bit 0 is the rightmost one, bit 7 the leftmost one
pub struct Set<'a> {
    bit: u8,
    dst: ID<'a>,
}

impl<'a> Set<'a> {
    pub fn new(bit: u8, dst: ID<'a>) -> Box<Self> { Box::new(Set { bit, dst }) }
}

impl<'a> Instruction<'a> for Set<'a> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8, _) => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *dst |= 1 << self.bit;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "set {}, {}", self.bit, self.dst)
    }
}
