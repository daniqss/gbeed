use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// rotate bits left between r8 and carry flag
///   ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┌─╂─   C   ←╂─╂─  b7  ←   ...  ←  b0  ←╂─┐
/// │ ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct Rla<'a> {
    carry: bool,
    a: &'a mut u8,
}

impl<'a> Rla<'a> {
    pub fn new(carry: bool, a: &'a mut u8) -> Box<Self> { Box::new(Self { carry, a }) }
}

impl<'a> Instruction<'a> for Rla<'a> {
    fn exec(&mut self) -> InstructionResult {
        let result = (*self.a << 1) | if self.carry { 1 } else { 0 };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(*self.a & 0b1000_0000 != 0),
        };
        *self.a = result;

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
        let mut a = 0b1000_0000;
        let mut instr = Rla::new(false, &mut a);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0000_0000);

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

        let mut instr = Rla::new(true, &mut a);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0111_0001);

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
