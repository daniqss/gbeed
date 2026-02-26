use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R8,
    },
    prelude::*,
};

/// Shift Left Arithmetically register r8.
///
/// ┏━ Flags ━┓ ┏━━━━━━━ r8 | [hl] ━━━━━━┓
/// ┃    C   ←╂─╂─   b7  ←  ...  ←  b0  ←╂─ 0
/// ┗━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━━┛
pub struct SlaR8 {
    dst: R8,
}
impl SlaR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}
impl Instruction for SlaR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let result = val << 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(val & 0b1000_0000 != 0),
        };
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("sla {}", self.dst) }
}

pub struct SlaPointedByHL;
impl SlaPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for SlaPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let result = val << 1;
        let flags = Flags {
            z: Some(result == 0),
            n: Some(false),
            h: Some(false),
            c: Some(val & 0b1000_0000 != 0),
        };
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(), flags))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "sla [hl]".to_string() }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::R16, Accessible16};

    use super::*;

    #[test]
    fn test_sla_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0000;
        let mut instr = SlaR8::new(R8::A);

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
    fn test_sla_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0001);
        let mut instr = SlaPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        assert_eq!(gb.read(addr), 0b0000_0010);

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
