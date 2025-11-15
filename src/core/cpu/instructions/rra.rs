use crate::core::cpu::{
    R8,
    flags::{CARRY_FLAG_MASK, Flags},
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// rotate bits left between r8 and carry flag
///   ┏━━━━━━━ A  ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └─────────────────────────────────┘
pub struct Rra<'a> {
    carry: u8,
    dst: ID<'a>,
}

impl<'a> Rra<'a> {
    pub fn new(carry: u8, dst: ID<'a>) -> Box<Self> { Box::new(Self { carry, dst }) }
}

impl<'a> Instruction<'a> for Rra<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8, reg) if *reg == R8::A => (r8, 1, 1),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = (*dst >> 1)
            | if self.carry & CARRY_FLAG_MASK != 0 {
                1 << 7
            } else {
                0
            };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b0000_0001 != 0),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "rr {}", self.dst)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::{R8, flags::Flags};

    use super::*;

    #[test]
    fn test_rr_no_carry() {
        let mut a = 0b0000_0001;
        let mut instr = Rra::new(0, ID::Reg8(&mut a, R8::A));

        let result = instr.exec().unwrap();
        assert_eq!(a, 0);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
    }

    #[test]
    fn test_rr_with_carry() {
        let mut a = 0b0011_1000;

        let mut instr = Rra::new(CARRY_FLAG_MASK, ID::Reg8(&mut a, R8::A));

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b1001_1100);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            }
        );
    }
}
