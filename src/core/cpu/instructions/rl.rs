use crate::core::cpu::{
    flags::{CARRY_FLAG_MASK, Flags, check_zero},
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// rotete bits left between r8 and carry flag
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─  b7  ←   ...  ←  b0  ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rl<'a> {
    carry: u8,
    dst: ID<'a>,
}

impl<'a> Rl<'a> {
    pub fn new(carry: u8, dst: ID<'a>) -> Box<Self> { Box::new(Rl { carry, dst }) }
}

impl<'a> Instruction<'a> for Rl<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Register8(r8, _) => (r8, 2, 2),
            ID::PointedByHL(bus, addr) => (&mut bus.borrow_mut()[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        // let mut flags = if *dst & 0b1000_0000 != 0 {
        //     CARRY_FLAG_MASK
        // } else {
        //     0
        // };
        let mut flags = Flags {
            z: None,
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b1000_0000 != 0),
        };

        *dst <<= 1;
        *dst |= if self.carry & CARRY_FLAG_MASK != 0 {
            0b0000_0001
        } else {
            0
        };

        flags.z = Some(check_zero(*dst));

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "rl {}", self.dst)
    }
}
