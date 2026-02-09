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

/// Shift Right Arithmetically register r8 (bit 7 of r8 is unchanged).
/// ┏━━━━━━ r8 ━━━━━━┓ ┏━ Flags ━┓
/// ┃ b7 → ... → b0 ─╂─╂→   C    ┃
/// ┗━━━━━━━━━━━━━━━━┛ ┗━━━━━━━━━┛
pub struct SraR8 {
    dst: R8,
}
impl SraR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}
impl Instruction for SraR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(self.dst);
        let last_bit = val & 0b1000_0000;
        let result = (val >> 1) | last_bit;
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
    fn disassembly(&self) -> String { format!("sra {}", self.dst) }
}

pub struct SraPointedByHL;
impl SraPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for SraPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let val = gb.read(gb.cpu.hl());
        let last_bit = val & 0b1000_0000;
        let result = (val >> 1) | last_bit;
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
    fn disassembly(&self) -> String { "sra [hl]".to_string() }
}

#[cfg(test)]
mod tests {
    use crate::core::{Accessible16, cpu::R16};

    use super::*;

    #[test]
    fn test_sra_r8() {
        let mut gb = Dmg::default();
        gb.cpu.a = 0b1000_0001;
        let mut instr = SraR8::new(R8::A);

        let result = instr.exec(&mut gb).unwrap();
        // SRA: bit 7 (1) is preserved. (1000 0001) >> 1 -> (0100 0000) | (1000 0000) -> 1100 0000. Carry is 1.
        assert_eq!(gb.cpu.a, 0b1100_0000);

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
    fn test_sra_pointed_by_hl() {
        let mut gb = Dmg::default();
        let addr = 0xC000;
        gb.store(R16::HL, addr);
        gb.write(addr, 0b0000_0000);
        let mut instr = SraPointedByHL::new();

        let result = instr.exec(&mut gb).unwrap();
        // SRA: bit 7 (0) is preserved. 0 >> 1 -> 0 | 0 -> 0. Carry is 0.
        assert_eq!(gb.read(addr), 0);

        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            }
        );
    }
}