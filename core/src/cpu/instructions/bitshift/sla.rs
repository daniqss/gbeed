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

/// Shift Left Arithmetically register r8.
///
/// ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┃    C   ←╂─╂─   b7  ←  ...  ←  b0  ←╂─ 0
/// ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛
#[derive(Debug, Default, Clone, Copy)]
pub struct SlaR8 {
    dst: R8,
}
impl SlaR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}
impl Instruction for SlaR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let result = val << 1;
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SlaFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sla {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SlaPointedByHL;
impl SlaPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SlaPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val << 1;
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SlaFlags::new(result, val).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "sla [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct SlaFlags {
    result: u8,
    dst: u8,
}

impl SlaFlags {
    fn new(result: u8, dst: u8) -> StaticBox<Self> { StaticBox::new(Self { result, dst }) }
}

impl LazyFlags for SlaFlags {
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

    use crate::{Accessible16, cpu::R16};

    #[test]
    fn test_sla_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = SlaR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(flags.carry());
    }

    #[test]
    fn test_sla_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0001);
        let mut instr = SlaPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0000_0010);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        let flags = result.flags.unwrap();
        assert!(!flags.zero());
        assert!(!flags.subtraction());
        assert!(!flags.half_carry());
        assert!(!flags.carry());
    }
}
