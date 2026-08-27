use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

/// rotate bits left
/// ┏━ Flags ━┓   ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┃    C   ←╂─┬─╂─   b7  ←  ...  ←  b0  ←╂─┐
/// ┗━━━━━━━━━┛ │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ │
///             └────────────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct Rlc<D: WritableOperand> {
    dst: D,
}
impl<D: WritableOperand> Rlc<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: WritableOperand> Instruction for Rlc<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old = self.dst.read(gb);
        let result = old.rotate_left(1);
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(result)),
                n: Some(false),
                h: Some(false),
                c: Some(old & 0b1000_0000 != 0),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("rlc {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::{
        Accessible,
        cpu::{R8, flags::Flags, instructions::PointedByHL},
    };

    use super::*;

    #[test]
    fn test_rlc_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = Rlc::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.cpu.a, 0b0000_0001);

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
    fn test_rlc_with_carry() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.cpu.set_hl(addr);
        gb.write(addr, 0b0011_1000);

        let mut instr = Rlc::new(PointedByHL);

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0111_0000);

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
