use crate::{
    Dmg,
    core::cpu::{
        R8,
        flags::Flags,
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
    },
};

/// Shift Left Arithmetically register r8.
///
/// ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┃    C   ←╂─╂─   b7  ←  ...  ←  b0  ←╂─ 0
/// ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛
pub struct Sla {
    dst: ID,
}

impl Sla {
    pub fn new(dst: ID) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for Sla {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(reg) if *reg != R8::F => (&mut gb[&*reg], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = *dst << 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b1000_0000 != 0),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "sla {}", self.dst)
    }
}
