use crate::{
    Dmg,
    core::cpu::{
        Reg,
        flags::Flags,
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
    },
};

/// Shift Right Logically register r8.
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
pub struct Srl {
    dst: ID,
}

impl Srl {
    pub fn new(dst: ID) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for Srl {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(reg) if *reg != Reg::F => (&mut gb[&*reg], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),
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
