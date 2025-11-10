use crate::core::cpu::{
    R8,
    flags::{Flags, check_zero},
    instructions::{
        Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
    },
};

/// rotate bits left between a and carry flag
/// ┏━ Flags ━┓   ┏━━━━━━━  a  ━━━━━━┓
/// ┃    C   ←╂─┬─╂─ b7 ← ... ← b0  ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━━┛ │
///             └──────────────────────┘
pub struct Rlca<'a> {
    dst: ID<'a>,
}

impl<'a> Rlca<'a> {
    pub fn new(dst: ID<'a>) -> Box<Self> { Box::new(Self { dst }) }
}

impl<'a> Instruction<'a> for Rlca<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Register8(r8, reg) if *reg == R8::A => (r8, 1, 1),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let last_bit = *dst & 0b1000_0000 != 0;
        let result = (*dst << 1) | if last_bit { 1 } else { 0 };
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(false),
            c: Some(last_bit),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "rlca {}", self.dst)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::{R8, flags::Flags};

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut a = 0b1000_0000;
        let mut instr = Rlca::new(ID::Register8(&mut a, R8::A));

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0000_0001);

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
    fn test_rl_with_carry() {
        let mut a = 0b0011_1000;

        let mut instr = Rlca::new(ID::Register8(&mut a, R8::A));

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0111_0000);

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
