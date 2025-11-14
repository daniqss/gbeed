use crate::core::cpu::{
    R8,
    flags::Flags,
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// Shift Right Logically register r8.
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
pub struct Srl<'a> {
    dst: ID<'a>,
}

impl<'a> Srl<'a> {
    pub fn new(dst: ID<'a>) -> Box<Self> { Box::new(Self { dst }) }
}

impl<'a> Instruction<'a> for Srl<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Register8(r8, reg) if *reg != R8::F => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = *dst >> 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b0000_0001 != 0),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "srl {}", self.dst)
    }
}
