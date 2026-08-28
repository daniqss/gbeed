use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, WritableOperand},
    },
    prelude::*,
};

/// Shift Right Logically register r8.
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
#[derive(Debug, Default, Clone, Copy)]
pub struct Srl<D: WritableOperand> {
    dst: D,
}
impl<D: WritableOperand> Srl<D> {
    pub fn new(dst: D) -> Self { Self { dst } }
}
impl<D: WritableOperand> Instruction for Srl<D> {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let old = self.dst.read(gb);
        let result = old >> 1;
        self.dst.write(gb, result);

        Ok(InstructionEffect::new(
            self.info(),
            Flags {
                z: Some(result == 0),
                n: Some(false),
                h: Some(false),
                c: Some(old & 0b0000_0001 != 0),
            },
        ))
    }
    fn info(&self) -> (u8, u8) { (2 + D::READ_CYCLES + D::WRITE_CYCLES, 2 + D::LEN) }
    fn disassembly(&self) -> String { format!("srl {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::{
        Accessible16,
        cpu::{R8, R16, instructions::PointedByHL},
    };

    use super::*;

    #[test]
    fn test_srl_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0001;
        let mut instr = Srl::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        // SRL: 0 -> b7. (1000 0001) >> 1 -> 0100 0000. Carry is 1.
        assert_eq!(gb.cpu.a, 0b0100_0000);

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
    fn test_srl_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0001);
        let mut instr = Srl::new(PointedByHL);

        let result = instr.exec(&mut gb).unwrap();
        // SRL: 0 -> b7. (0000 0001) >> 1 -> 0. Carry is 1.
        assert_eq!(gb.read(addr), 0);

        assert_eq!(result.cycles, 4);
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
}
