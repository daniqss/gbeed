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
fn rrc(value: u8) -> u8 { (value >> 1) | ((value & 1) << 7) }

/// rotate bits right
///   ┏━━━━━━━ r8 | [hl] ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→  b7  →  ...  →  b0   ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct RrcR8 {
    dst: R8,
}

impl RrcR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for RrcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = rrc(r8);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RrcFlags::new(result, r8).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("rrc {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RrcPointedByHL;

impl RrcPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}

impl Instruction for RrcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = rrc(val);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(RrcFlags::new(result, val).into()),
        ))
    }

    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "rrc [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct RrcFlags {
    result: u8,
    dst: u8,
}

impl RrcFlags {
    fn new(result: u8, dst: u8) -> StaticBox<Self> { StaticBox::new(Self { result, dst }) }
}

impl LazyFlags for RrcFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { self.dst & 0b0000_0001 != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Accessible, cpu::R8};

    #[test]
    fn test_rrc_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = RrcR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1000_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_rrc_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.cpu.set_hl(addr);
        gb.cpu.set_carry();
        gb.write(addr, 0b0011_1000);

        let mut instr = RrcPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0001_1100);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
