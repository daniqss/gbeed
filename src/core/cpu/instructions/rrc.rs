use crate::{
    Dmg,
    core::{
        Accessible,
        cpu::{
            R8,
            flags::{Flags, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
    },
};

#[inline(always)]
fn rrc_flags(result: u8, dst: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(false),
        c: Some(dst & 0b0000_0001 != 0),
    }
}

#[inline(always)]
fn rrc(value: u8) -> u8 { (value >> 1) | ((value & 1) << 7) }

/// rotate bits right
///   ┏━━━━━━━ r8 | [hl] ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→  b7  →  ...  →  b0   ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct RrcR8 {
    dst: R8,
}

impl RrcR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for RrcR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = rrc(r8);
        let flags = rrc_flags(result, r8);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("rrc {}", self.dst) }
}

pub struct RrcPointedByHL;

impl RrcPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self {}) }
}

impl Instruction for RrcPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = rrc(val);
        let flags = rrc_flags(result, val);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), flags))
    }

    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("rrc [hl]") }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Accessible,
        cpu::{R8, flags::Flags},
    };

    use super::*;

    #[test]
    fn test_rrc_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = RrcR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b1000_0000);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
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
    fn test_rrc_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xFF00;
        gb.write(addr, 0b0011_1000);

        let mut instr = RrcPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0001_1100);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
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