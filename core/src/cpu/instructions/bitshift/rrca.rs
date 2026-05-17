use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn rrca(value: u8) -> u8 { (value >> 1) | ((value & 1) << 7) }

/// rotate bits right A
///   ┏━━━━━━━ A ━━━━━━━┓   ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─┬─╂→   C    ┃
/// │ ┗━━━━━━━━━━━━━━━━━┛ │ ┗━━━━━━━━━┛
/// └─────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct Rrca;

impl Rrca {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Rrca {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let dst = gb.cpu.a;
        gb.cpu.a = rrca(dst);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RrcaFlags::new(dst).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "rrca".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct RrcaFlags {
    dst: u8,
}

impl RrcaFlags {
    fn new(dst: u8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl LazyFlags for RrcaFlags {
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
    fn test_rrca_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = Rrca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1000_0000);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_rrca_with_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0011_1000;

        let mut instr = Rrca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0001_1100);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
