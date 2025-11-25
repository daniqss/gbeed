use std::fmt::Write;

use super::InstructionTarget as IT;
use crate::{
    Dmg,
    core::cpu::{
        R8,
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionError, InstructionResult},
    },
};

/// Substraction with carry instruction
pub struct Sbc {
    carry: bool,
    subtrahend: IT,
}

impl Sbc {
    pub fn new(carry: bool, subtrahend: IT) -> Box<Self> { Box::new(Sbc { carry, subtrahend }) }
}

impl Instruction for Sbc {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (subtrahend, cycles, len) = match &self.subtrahend {
            IT::Reg8(val, reg) if *reg != R8::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform substraction
        let (result, did_borrow_sub) = gb.cpu.a.overflowing_sub(subtrahend);
        let (result, did_borrow_cy) = result.overflowing_sub(if self.carry { 1 } else { 0 });

        // calculate new flags
        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(gb.cpu.a, subtrahend)),
            c: Some(did_borrow_sub || did_borrow_cy),
        };

        gb.cpu.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "sbc a,{}", self.subtrahend)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_sbc_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        let subtrahend = IT::Imm8(19);

        let mut sbc = Sbc::new(true, subtrahend);
        let result = sbc.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(true),
                h: Some(false),
                c: Some(false),
            }
        );
    }

    #[test]
    fn test_sbc_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        let subtrahend = IT::Reg8(0b0000_00011, R8::B);

        let mut sbc = Sbc::new(false, subtrahend);
        let result = sbc.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0b0000_1101);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(true),
                c: Some(false),
            }
        );
    }

    #[test]
    fn test_sbc_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        let subtrahend = IT::PointedByHL(0x20);

        let mut sbc = Sbc::new(false, subtrahend);
        let result = sbc.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(true),
            }
        );
    }
}
