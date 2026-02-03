use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        Reg,
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

/// Subtraction instruction
/// Subtracts the value of the specified target from register A
/// Always sets the subtraction flag, sets zero flag if result is zero, and sets half-carry and carry flags if there is a borrow in bits 4
pub struct Sub {
    subtrahend: IT,
}

impl Sub {
    pub fn new(subtrahend: IT) -> Box<Self> { Box::new(Sub { subtrahend }) }
}

impl Instruction for Sub {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (subtrahend, cycles, len) = match &self.subtrahend {
            IT::Reg8(val, reg) if *reg != Reg::F => (*val, 1, 1),
            IT::PointedByHL(value) => (*value, 2, 1),
            IT::Imm8(n8) => (*n8, 2, 2),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // perform the subtraction
        let (result, did_borrow) = gb.cpu.a.overflowing_sub(subtrahend);

        let flags = Flags {
            z: Some(check_zero(result)),
            n: Some(true),
            h: Some(check_borrow_hc(gb.cpu.a, subtrahend)),
            c: Some(did_borrow),
        };

        gb.cpu.a = result;

        Ok(InstructionEffect::new(cycles, len, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> {
        write!(w, "sub a,{}", self.subtrahend)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::cpu::flags::Flags;

    use super::*;

    #[test]
    fn test_sub_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        let mut instr = Sub::new(IT::Imm8(20));

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0);

        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
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
    fn test_sub_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        let subtrahend = IT::Reg8(0b0000_0001, Reg::B);

        let mut instr = Sub::new(subtrahend);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0x0F);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
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
    fn test_sub_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        let subtrahend = IT::PointedByHL(0x20);

        let mut instr = Sub::new(subtrahend);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0xF0);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 1);
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
