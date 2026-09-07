use crate::{
    cpu::{
        flags::{Flags, check_zero},
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

/// rotate bits right between r8 and carry flag
///   ┏━━━━━━━ r8 | [hl] ━━━━━━┓ ┏━ Flags ━┓
/// ┌─╂->  b7  ->  ...  -> b  ─╂─╂->  C   ─╂─┐
/// │ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛ │
/// └────────────────────────────────────────┘
#[derive(Debug, Default, Clone, Copy)]
pub struct Rr<D: WritableOperand> {
    dst: D,
}
impl<D: WritableOperand> Rr<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: WritableOperand> Instruction for Rr<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old = self.dst.read(gb);
        let result = (old >> 1) | if gb.cpu.carry() { 1 << 7 } else { 0 };
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(check_zero(result)),
                n: Some(false),
                h: Some(false),
                c: Some(old & 0b0000_0001 != 0),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("rr {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::{
        Accessible,
        cpu::{R8, flags::Flags, instructions::PointedByHL},
    };

    use super::*;

    #[test]
    fn test_rr_no_carry() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b0000_0001;
        gb.cpu.clear_carry();
        let mut instr = Rr::new(R8::A);

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
        let addr = 0xC000;
        gb.cpu.set_hl(addr);
        gb.cpu.set_carry();
        gb.write(addr, 0b0011_1000);

        let mut instr = Rr::new(PointedByHL);

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
