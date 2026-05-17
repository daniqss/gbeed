use crate::{
    cpu::{
        flags::{CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn rlca(value: u8) -> u8 { value.rotate_left(1) }

/// rotate bits left a
/// ┏━ Flags ━┓   ┏━━━━━━━  a  ━━━━━━┓
/// ┃    C   ←╂─┬─╂─ b7 ← ... ← b0  ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━━┛ │
///             └──────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct Rlca;

impl Rlca {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for Rlca {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let dst = gb.cpu.a;
        gb.cpu.a = rlca(dst);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RlcaFlags::new(dst).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "rlca".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct RlcaFlags {
    dst: u8,
}

impl RlcaFlags {
    fn new(dst: u8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl LazyFlags for RlcaFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { false }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { self.dst & 0b1000_0000 != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlca_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;

        let mut instr = Rlca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0001);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_rlca_with_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0011_1000;

        let mut instr = Rlca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0111_0000);

        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
