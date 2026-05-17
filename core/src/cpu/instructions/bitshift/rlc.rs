use crate::{
    cpu::{
        R8,
        flags::{
            CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK,
            check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[inline(always)]
fn rlc(value: u8) -> u8 { value.rotate_left(1) }

/// rotate bits left
/// ┏━ Flags ━┓   ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┃    C   ←╂─┬─╂─   b7  ←  ...  ←  b0  ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
///             └────────────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct RlcR8 {
    dst: R8,
}

impl RlcR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for RlcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = rlc(r8);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RlcFlags::new(result, r8).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("rlc {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RlcPointedByHL;

impl RlcPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for RlcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = rlc(val);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RlcFlags::new(result, val).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "rlc [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct RlcFlags {
    result: u8,
    dst: u8,
}

impl RlcFlags {
    fn new(result: u8, dst: u8) -> StaticBox<Self> { StaticBox::new(Self { result, dst }) }
}

impl LazyFlags for RlcFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { self.dst & 0b1000_0000 != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Accessible, cpu::R8};

    #[test]
    fn test_rlc_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = RlcR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0001);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_rlc_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.cpu.set_hl(addr);
        gb.write(addr, 0b0011_1000);

        let mut instr = RlcPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0111_0000);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
