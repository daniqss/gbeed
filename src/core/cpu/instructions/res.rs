use crate::core::cpu::instructions::{
    Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
};

/// Sets bit u3 in register r8 to 0. Bit 0 is the rightmost one, bit 7 the leftmost one
pub struct Res<'a> {
    bit: u8,
    target: ID<'a>,
}

impl<'a> Res<'a> {
    pub fn new(bit: u8, target: ID<'a>) -> Box<Self> { Box::new(Res { bit, target }) }
}

impl<'a> Instruction<'a> for Res<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (target, cycles, len): (&mut u8, u8, u8) = match &mut self.target {
            ID::Register8(r8, _) => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        *target &= !(1 << self.bit);

        Ok(InstructionEffect::new(cycles, len, None))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "res {}, {}", self.bit, self.target)
    }
}
