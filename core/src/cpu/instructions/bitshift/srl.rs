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

/// Shift Right Logically register r8.
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
#[derive(Debug, Default, Clone, Copy)]
pub struct SrlR8 {
    dst: R8,
}
impl SrlR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}
impl Instruction for SrlR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let result = val >> 1;
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SrlFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("srl {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SrlPointedByHL;
impl SrlPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SrlPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val >> 1;
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SrlFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "srl [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct SrlFlags {
    result: u8,
    dst: u8,
}

impl SrlFlags {
    fn new(result: u8, dst: u8) -> StaticBox<Self> { StaticBox::new(Self { result, dst }) }
}

impl LazyFlags for SrlFlags {
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
    fn test_srl_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0001;
        let mut instr = SrlR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        // SRL: 0 -> b7. (1000 0001) >> 1 -> 0100 0000. Carry is 1.
        assert_eq!(gb.cpu.a, 0b0100_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_srl_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0001);
        let mut instr = SrlPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        // SRL: 0 -> b7. (0000 0001) >> 1 -> 0. Carry is 1.
        assert_eq!(gb.read(addr), 0);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }
}
