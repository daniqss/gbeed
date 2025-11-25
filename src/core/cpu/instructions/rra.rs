use crate::core::cpu::{
    flags::Flags,
    instructions::{Instruction, InstructionEffect, InstructionResult},
};

/// rotate bits left between r8 and carry flag
///   ┏━━━━━━━ A  ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └─────────────────────────────────┘
pub struct Rra<'a> {
    carry: bool,
    a: &'a mut u8,
}

impl<'a> Rra<'a> {
    pub fn new(carry: bool, a: &'a mut u8) -> Box<Self> { Box::new(Self { carry, a }) }
}

impl<'a> Instruction<'a> for Rra<'a> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = (*self.a >> 1) | if self.carry { 1 << 7 } else { 0 };
        let flags = Flags {
            z: Some(false),
            n: Some(false),
            h: Some(false),
            c: Some(*self.a & 0b0000_0001 != 0),
        };
        *self.a = result;

        Ok(InstructionEffect::new(1, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> { write!(w, "rra") }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_rr_no_carry() {
        let mut a = 0b0000_0001;
        let mut instr = Rra::new(false, &mut a);

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

        let mut instr = Rra::new(true, &mut a);

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
