use crate::{
    Dmg,
    core::{
        Accessible,
        cpu::{
            R8,
            flags::Flags,
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
    },
};

/// Shift Right Logically register r8.
///    ┏━━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// 0 ─╂→ b7 → ... → b0 ─╂─╂→   C    ┃
///    ┗━━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
pub struct SrlR8 {
    dst: R8,
}
impl SrlR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}
impl Instruction for SrlR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let result = val >> 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(val & 0b0000_0001 != 0),
        };
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("srl {}", self.dst) }
}

pub struct SrlPointedByHL;
impl SrlPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for SrlPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val >> 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(val & 0b0000_0001 != 0),
        };
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "srl [hl]".to_string() }
}

#[cfg(test)]
mod tests {
    use crate::core::{Accessible16, cpu::R16};

    use super::*;

    #[test]
    fn test_srl_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0001;
        let mut instr = SrlR8::new(R8::A);

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
        let mut instr = SrlPointedByHL::new();

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
