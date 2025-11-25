use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

/// rotate bits left between r8 and carry flag
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─  b7  ←   ...  ←  b0  ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rla {
    carry: bool,
}

impl Rla {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}

impl Instruction for Rla {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = (gb.cpu.a << 1) | if self.carry { 1 } else { 0 };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(gb.cpu.a & 0b1000_0000 != 0),
        };
        gb.cpu.a = result;

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "rla") }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = Rla::new(false);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0000);

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

        let mut instr = Rla::new(true);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0111_0001);

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
