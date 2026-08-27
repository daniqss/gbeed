use crate::{
    cpu::{
        flags::{Flags, check_borrow_hc, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, Operand},
    },
    prelude::*,
};

/// Subtraction instruction
/// Subtracts the given operand from register A
#[derive(Debug, Default, Clone, Copy)]
pub struct Sub<S: Operand> {
    src: S,
}
impl<S: Operand> Sub<S> {
    pub fn new(src: S) -> Self { Self { src } }
}
impl<S: Operand> Instruction for Sub<S> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old_a = gb.cpu.a;
        let subtrahend = self.src.read(gb);
        gb.cpu.a = old_a.wrapping_sub(subtrahend);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(gb.cpu.a)),
                n: Some(true),
                h: Some(check_borrow_hc(old_a, subtrahend)),
                c: Some(old_a < subtrahend),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (1 + S::READ_CYCLES, 1 + S::LEN) }
    fn disassembly(&self) -> String { format!("sub a,{}", self.src) }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{
        R8,
        flags::Flags,
        instructions::{Imm8, PointedByHL},
    };

    use super::*;

    #[test]
    fn test_sub_zero_result() {
        let mut gb = Dmg::default();
        gb.cpu.a = 20;
        let mut instr = Sub::new(Imm8(20));

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
        gb.cpu.b = 0b0000_0001;

        let mut instr = Sub::new(R8::B);
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
        gb.write(0xC020, 0x20);
        gb.cpu.h = 0xC0;
        gb.cpu.l = 0x20;

        let mut instr = Sub::new(PointedByHL);
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
