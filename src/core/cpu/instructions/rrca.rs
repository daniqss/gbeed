use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
};

#[inline(always)]
fn rrca_flags(dst: u8) -> Flags {
    Flags {
        z: Some(false),
        n: Some(false),
        h: Some(false),
        c: Some(dst & 0b0000_0001 != 0),
    }
}

#[inline(always)]
fn rrca(value: u8) -> u8 { (value >> 1) | ((value & 1) << 7) }

/// rotate bits right A
///   ┏━━━━━━━ A ━━━━━━━┓   ┏━ Flags ━┓
/// ┌─╂→ b7 → ... → b0 ─╂─┬─╂→   C    ┃
/// │ ┗━━━━━━━━━━━━━━━━━┛ │ ┗━━━━━━━━━┛
/// └─────────────────────┘
pub struct Rrca;

impl Rrca {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for Rrca {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let result = rrca(gb.cpu.a);
        let flags = rrca_flags(gb.cpu.a);
        gb.cpu.a = result;

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { format!("rrca") }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

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
    fn test_rrca_with_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0011_1000;

        let mut instr = Rrca::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0001_1100);

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
