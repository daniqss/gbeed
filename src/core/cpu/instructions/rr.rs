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
fn rr_flags(result: u8, dst: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(false),
        h: Some(false),
        c: Some(dst & 0b0000_0001 != 0),
    }
}

#[inline(always)]
fn rr(value: u8, carry: bool) -> u8 { (value >> 1) | if carry { 1 << 7 } else { 0 } }

/// rotate bits right between r8 and carry flag
///   ┏━━━━━━━ r8 | [hl] ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂→  b7  →  ...  →  b0   ─╂─╂→   C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
pub struct RrR8 {
    carry: bool,
    dst: R8,
}
impl RrR8 {
    pub fn new(carry: bool, dst: R8) -> Box<Self> { Box::new(Self { carry, dst }) }
}
impl Instruction for RrR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = rr(r8, self.carry);
        let flags = rr_flags(result, r8);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("rr {}", self.dst) }
}

pub struct RrPointedByHL {
    carry: bool,
}
impl RrPointedByHL {
    pub fn new(carry: bool) -> Box<Self> { Box::new(Self { carry }) }
}
impl Instruction for RrPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = rr(val, self.carry);
        let flags = rr_flags(result, val);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("rr [hl]") }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Accessible,
        cpu::{R8, flags::Flags},
    };

    use super::*;

    #[test]
    fn test_rr_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        let mut instr = RrR8::new(false, R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
    }

    #[test]
    fn test_rr_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xFF00;
        gb.write(addr, 0b0011_1000);

        let mut instr = RrPointedByHL::new(true);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b1001_1100);

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
