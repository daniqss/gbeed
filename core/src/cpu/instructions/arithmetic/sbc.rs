use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

/// Subtraction with carry instruction
/// Subtracts the given operand from register A, and the carry flag
#[derive(Debug, Default, Clone, Copy)]
pub struct Sbc<S: Operand> {
    src: S,
}
impl<S: Operand> Sbc<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for Sbc<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let val = self.src.read(gb);
        let carry = if gb.cpu.carry() { 1 } else { 0 };
        gb.cpu.a = old_a.wrapping_sub(val).wrapping_sub(carry);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(true),
                h: Some((old_a & 0xF) < (val & 0xF) + carry),
                c: Some((old_a as u16) < (val as u16) + (carry as u16)),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("sbc a,{}", self.src) }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        R8,
        instructions::{Imm8, PointedByHL},
    };

    use super::*;

    #[test]
    fn test_sbc_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        gb.cpu.set_carry();
        let mut instr = Sbc::new(Imm8(19));

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
    fn test_sbc_set_half_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0001_0000;
        gb.cpu.b = 0b0000_0011;
        gb.cpu.clear_carry();

        let mut instr = Sbc::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 0b0000_1101);
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
    fn test_sbc_set_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0x10;
        gb.write(0xC020, 0x20);
        gb.cpu.h = 0xC0;
        gb.cpu.l = 0x20;
        gb.write(R8::F, 0);

        let mut instr = Sbc::new(PointedByHL);
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

    #[test]
    fn test_sbc_with_carry_flag() {
        let mut gb = Dmg::default();
        gb.cpu.a = 10;
        gb.cpu.b = 3;
        gb.cpu.set_carry();

        let mut instr = Sbc::new(R8::B);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 6);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(false),
                c: Some(false),
            }
        );

        gb.cpu.a = 5;
        gb.cpu.set_carry();
        let mut instr = Sbc::new(Imm8(5));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 255);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(false),
                n: Some(true),
                h: Some(true),
                c: Some(true),
            }
        );
    }
}
