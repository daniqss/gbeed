use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn rra_flags(dst: u8) -> Flags {
    Flags {
        z: Some(false),
        n: Some(false),
        h: Some(false),
        c: Some(dst & 0b0000_0001 != 0),
    }
}

#[inline(always)]
fn rra(value: u8, carry: bool) -> u8 { (value >> 1) | if carry { 1 << 7 } else { 0 } }

/// rotate bits right between A and carry flag
///   ┏━━━━━━━ A ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └─────────────────────────────────┘
pub struct Rra {
    carry: bool,
}

impl Rra {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}

impl Instruction for Rra {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = rra(gb.cpu.a, self.carry);
        let flags = rra_flags(gb.cpu.a);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("rra") }
}

#[cfg(test)]
mod tests {
    use crate::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_rra_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = Rra::new(false);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
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
    fn test_rra_with_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0011_1000;

        let mut instr = Rra::new(true);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1001_1100);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
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
