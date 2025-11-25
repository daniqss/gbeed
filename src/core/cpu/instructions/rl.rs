use crate::{
    Dmg,
    core::cpu::{
        flags::{Flags, check_zero},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
    },
};

/// rotate bits left between r8 and carry flag
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─  b7  ←   ...  ←  b0  ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rl {
    carry: bool,
    dst: ID,
}

impl Rl {
    pub fn new(carry: bool, dst: ID) -> Box<Self> { Box::new(Rl { carry, dst }) }
}

impl Instruction for Rl {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8) => (&mut gb[&*r8], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = (*dst << 1) | if self.carry { 1 } else { 0 };
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(false),
            c: Some(*dst & 0b1000_0000 != 0),
        };
        *dst = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(w, "rl {}", self.dst)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::{R8, flags::Flags};

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = Rl::new(false, ID::Reg8(R8::A));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
    }

    #[test]
    fn test_rl_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xFF00;
        gb[addr] = 0b0011_1000;

        let mut instr = Rl::new(true, ID::PointedByHL(gb.cpu.hl()));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb[addr], 0b0111_0001);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len, 2);
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
