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
pub struct Rr {
    carry: bool,
    dst: ID,
}

impl Rr {
    pub fn new(carry: bool, dst: ID) -> Box<Self> { Box::new(Rr { carry, dst }) }
}

impl Instruction for Rr {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (dst, cycles, len): (&mut u8, u8, u8) = match &mut self.dst {
            ID::Reg8(r8) => (&mut gb[&*r8], 2, 2),
            ID::PointedByHL(addr) => (&mut gb[*addr], 4, 2),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        let result = (*dst >> 1) | if self.carry { 1 << 7 } else { 0 };
        let flags = Flags {
            z: Some(check_zero(result)),
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
    use crate::core::cpu::{Reg, flags::Flags};

    use super::*;

    #[test]
    fn test_rr_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = Rr::new(false, ID::Reg8(Reg::A));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
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
    fn test_rr_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xFF00;
        let value = 0b0011_1000;
        gb[addr] = value;

        let mut instr = Rr::new(true, ID::PointedByHL(addr));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb[addr], 0b1001_1100);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
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
