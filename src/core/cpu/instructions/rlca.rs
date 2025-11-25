use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// rotate bits left between a and carry flag
/// ┏━ Flags ━┓   ┏━━━━━━━  a  ━━━━━━┓
/// ┃    C   ←╂─┬─╂─ b7 ← ... ← b0  ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━━┛ │
///             └──────────────────────┘
pub struct Rlca {}

impl Rlca {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Rlca {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let last_bit = gb.cpu.a & 0b1000_0000 != 0;
        let result = (gb.cpu.a << 1) | if last_bit { 1 } else { 0 };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(last_bit),
        };
        gb.cpu.a = result;

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "rlca") }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;

        let mut instr = Rlca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0001);

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
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0011_1000;

        let mut instr = Rlca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0111_0000);

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
