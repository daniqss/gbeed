use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition},
    },
    prelude::*,
};

/// relative jump to the given address
/// it can get a condition to jump only if the condition is met
/// the condition is based on carry and zero flags
/// the address is encoded as a signed 8 bit immediate value
pub struct Jr {
    pub jc: JumpCondition,
    pub offset: u8,
}

impl Jr {
    pub fn new(jc: JumpCondition, offset: u8) -> Box<Self> { Box::new(Self { jc, offset }) }
}

impl Instruction for Jr {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let should_jump = self.jc.should_jump();
        gb.cpu.pc = gb.cpu.pc.wrapping_add(2);

        if should_jump {
            // cast i8 offset to u16 to perform addition
            // offset is relative to the next instruction
            let result = if self.offset & 0x80 != 0 {
                let mut e8: u8 = !self.offset;
                e8 = e8.wrapping_add(1);

                gb.cpu.pc.wrapping_sub(e8 as u16)
            } else {
                gb.cpu.pc.wrapping_add(self.offset as u16)
            };
            gb.cpu.pc = result;
            Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
        } else {
            Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
        }
    }
    fn info(&self) -> (u8, u8) {
        if self.jc.should_jump() {
            (3, 2)
        } else {
            (2, 2)
        }
    }
    fn disassembly(&self) -> String { format!("jr {}{}", self.jc, self.offset as i8) }
}

#[cfg(test)]
mod test {
    use crate::cpu::{
        flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::JumpCondition as JC,
    };

    use super::*;

    #[test]
    fn test_jr_n16() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;
        let e8: u8 = 0x07;
        gb.write(pc + 1, e8 as u8);

        let mut instr = Jr::new(JC::None, e8);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc.wrapping_add((e8 + 2) as u16));
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_jr_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;

        // e8 = 0xFA = -6
        let e8: u8 = 0xFA;
        gb.write(pc + 1, e8);

        // zero flag is not set, so it should jump
        gb.cpu.f = !ZERO_FLAG_MASK;

        let mut instr = Jr::new(JC::NotZero(gb.cpu.not_zero()), e8);
        let result = instr.exec(&mut gb).unwrap();

        // interpret e8 as signed i8
        let offset = e8 as i8 as i16;
        let expected_pc = (pc as i16 + 2 + offset) as u16;

        assert_eq!(gb.cpu.pc, expected_pc);
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_jr_no_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;
        let e8: u8 = 0x64;
        gb.write(pc + 1, e8 as u8);
        // carry is not set, so it should not jump
        gb.cpu.f = !CARRY_FLAG_MASK;

        let mut instr = Jr::new(JC::Carry(gb.cpu.carry()), e8);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc + 2);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len(), 2);
    }
}
