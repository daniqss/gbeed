use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// rotate bits left between A and carry flag
///   ┏━━━━━━━ A ━━━━━━━┓   ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─┬─╂→   C    ┃
/// │ ┗━━━━━━━━━━━━━━━━━┛ │ ┗━━━━━━━━━┛
/// └─────────────────────┘
pub struct Rrca<'a> {
    a: &'a mut u8,
}

impl<'a> Rrca<'a> {
    pub fn new(a: &'a mut u8) -> Box<Self> { Box::new(Self { a }) }
}

impl<'a> Instruction<'a> for Rrca<'a> {
    fn exec(&mut self) -> InstructionResult {
        let first_bit = *self.a & 0b0000_0001 != 0;
        let result = (*self.a >> 1) | if first_bit { 0b1000_0000 } else { 0 };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(first_bit),
        };
        *self.a = result;

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "rrca") }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_rl_no_carry() {
        let mut a = 0b0000_0001;
        let mut instr = Rrca::new(&mut a);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b1000_0000);

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

        let mut instr = Rrca::new(&mut a);

        let result = instr.exec().unwrap();
        assert_eq!(a, 0b0001_1100);

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
