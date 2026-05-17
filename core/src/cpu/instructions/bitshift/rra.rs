use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn rra(value: u8, carry: bool) -> u8 { (value >> 1) | if carry { 1 << 7 } else { 0 } }

/// rotate bits right between A and carry flag
///   ┏━━━━━━━ A ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └─────────────────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct Rra {
    carry: bool,
}

impl Rra {
    pub fn new(carry: bool) -> StaticBox<Self> { StaticBox::new(Self { carry }) }
}

impl Instruction for Rra {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let dst = gb.cpu.a;
        gb.cpu.a = rra(dst, self.carry);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RraFlags::new(dst).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "rra".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct RraFlags {
    dst: u8,
}

impl RraFlags {
    fn new(dst: u8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl LazyFlags for RraFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { false }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { self.dst & 0b0000_0001 != 0 }
}

#[cfg(test)]
mod tests {
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
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
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
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
