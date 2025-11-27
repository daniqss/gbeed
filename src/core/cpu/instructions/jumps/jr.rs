use std::fmt::Write;

use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{
            Instruction, InstructionEffect, InstructionError, InstructionResult, InstructionTarget as IT,
        },
    },
};

/// relative jump to the given address
/// it can get a condition to jump only if the condition is met
/// the condition is based on carry and zero flags
/// the address is encoded as a signed 8 bit immediate value
pub struct Jr {
    pub jump: IT,
}

impl Jr {
    pub fn new(jump: IT) -> Box<Self> { Box::new(Self { jump }) }
}

impl Instruction for Jr {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (offset, cycles, len) = match &self.jump {
            IT::JumpToImm8(cc, offset) => {
                let should_jump = cc.should_jump();

                if should_jump {
                    // cast i8 offset to u16 to perform addition
                    // offset is relative to the next instruction
                    let offset = (*offset as i16 as u16).wrapping_add(2);
                    (offset, 3, 0)
                } else {
                    (0, 2, 2)
                }
            }

            _ => return Err(InstructionError::MalformedInstruction),
        };

        gb.cpu.pc = gb.cpu.pc.wrapping_add(offset);

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "jr {}", self.jump) }
}

#[cfg(test)]
mod test {
    use crate::core::cpu::{
        flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::JumpCondition as JC,
    };

    use super::*;

    #[test]
    fn test_jr_n16() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;
        let e8: i8 = -7;
        gb[pc + 1] = e8 as u8;

        let mut instr = Jr::new(IT::JumpToImm8(JC::None, e8));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc.wrapping_add((e8 + 2) as u16));
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len, 0);
    }

    #[test]
    fn test_jr_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;
        let e8: i8 = 5;
        gb[pc + 1] = e8 as u8;
        // zero flag is not set, so it should jump
        gb.cpu.f = !ZERO_FLAG_MASK;

        let mut instr = Jr::new(IT::JumpToImm8(JC::NotZero(gb.cpu.not_zero()), e8));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc.wrapping_add((e8 + 2) as u16));
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len, 0);
    }

    #[test]
    fn test_jr_no_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        let pc = gb.cpu.pc;
        let e8: i8 = -3;
        gb[pc + 1] = e8 as u8;
        // carry is not set, so it should not jump
        gb.cpu.f = !CARRY_FLAG_MASK;

        let mut instr = Jr::new(IT::JumpToImm8(JC::Carry(gb.cpu.carry()), e8));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc);
        assert_eq!(result.cycles, 2);
        assert_eq!(result.len, 2);
    }
}
