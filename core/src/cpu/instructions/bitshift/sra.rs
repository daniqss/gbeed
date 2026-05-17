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

/// Shift Right Arithmetically register r8 (bit 7 of r8 is unchanged).
/// ┏━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// ┃ b7 → ... → b0 ─╂─╂→   C    ┃
/// ┗━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
#[derive(Debug, Default, Clone, Copy)]
pub struct SraR8 {
    dst: R8,
}
impl SraR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}
impl Instruction for SraR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let last_bit = val & 0b1000_0000;
        let result = (val >> 1) | last_bit;
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SraFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sra {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SraPointedByHL;
impl SraPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SraPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let last_bit = val & 0b1000_0000;
        let result = (val >> 1) | last_bit;
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SraFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "sra [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct SraFlags {
    result: u8,
    dst: u8,
}

impl SraFlags {
    fn new(result: u8, dst: u8) -> StaticBox<Self> { StaticBox::new(Self { result, dst }) }
}

impl LazyFlags for SraFlags {
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

    use crate::{Accessible16, cpu::R16};

    #[test]
    fn test_sra_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0001;
        let mut instr = SraR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        // SRA: bit 7 (1) is preserved. (1000 0001) >> 1 -> (0100 0000) | (1000 0000) -> 1100 0000. Carry is 1.
        assert_eq!(gb.cpu.a, 0b1100_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_sra_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0000);
        let mut instr = SraPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        // SRA: bit 7 (0) is preserved. 0 >> 1 -> 0 | 0 -> 0. Carry is 0.
        assert_eq!(gb.read(addr), 0);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
