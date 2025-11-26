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
///   ┏━━━━━━━ r8 | [hl] ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→  b7  →  ...  →  b0   ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rrc {
    dst: ID,
}

impl Rrc {
    pub fn new(dst: ID) -> Box<Self> { Box::new(Rrc { dst }) }
}

impl Instruction for Rrc {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(reg) => (&mut gb[&*reg], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let first_bit = *dst & 0b0000_0001 != 0;
        let result = (*dst >> 1) | if first_bit { 0b1000_0000 } else { 0 };
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(false),
            h: Some(false),
            c: Some(first_bit),
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
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = Rrc::new(ID::Reg8(R8::A));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1000_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
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
        let mut gb = Dmg::default();
        let addr = 0xFF00;
        let value = 0b0011_1000;
        gb[addr] = value;

        let mut instr = Rrc::new(ID::PointedByHL(addr));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb[addr], 0b0001_1100);

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
