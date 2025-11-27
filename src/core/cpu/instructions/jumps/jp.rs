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

/// jump to the given address
/// it can get a condition to jump only if the condition is met
/// this condition is based on carry and zero flags
pub struct Jp {
    pub jump: IT,
}

impl Jp {
    pub fn new(jump: IT) -> Box<Self> { Box::new(Self { jump }) }
}

impl Instruction for Jp {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let (addr, cycles, len) = match &self.jump {
            IT::JumpToImm16(cc, addr) => {
                let should_jump = cc.should_jump();

                let addr = if should_jump { *addr } else { gb.cpu.pc };
                let cycles = if should_jump { 4 } else { 3 };
                // TODO: return len as 0 if jumped?
                let len = if should_jump { 0 } else { 3 };

                (addr, cycles, len)
            }

            IT::JumpToHL(addr) => (*addr, 1, 0),

            _ => return Err(InstructionError::MalformedInstruction),
        };

        gb.cpu.pc = addr;

        Ok(InstructionEffect::new(cycles, len, Flags::none()))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "jp {}", self.jump) }
}

#[cfg(test)]
mod test {
    use crate::core::{
        cpu::{
            R16,
            flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
            instructions::JumpCondition as JC,
        },
        memory::Accessable,
    };

    use super::*;

    #[test]
    fn test_jump_to_hl() {
        let mut gb = Dmg::default();
        gb.write16(&R16::HL, 0x1234);

        let mut instr = Jp::new(IT::JumpToHL(gb.cpu.hl()));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x1234);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len, 0);
    }

    #[test]
    fn test_jump_n16() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        gb.write16(gb.cpu.pc + 1, 0x200);

        let mut instr = Jp::new(IT::JumpToImm16(JC::None, gb.read16(gb.cpu.pc + 1)));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x200);
        assert_eq!(result.cycles, 4);
        assert_eq!(result.len, 0);
    }

    #[test]
    fn test_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        gb.cpu.f = ZERO_FLAG_MASK;
        gb.write16(gb.cpu.pc + 1, 0x200);

        let mut instr = Jp::new(IT::JumpToImm16(JC::Zero(gb.cpu.zero()), gb.read16(gb.cpu.pc + 1)));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x200);
        assert_eq!(result.cycles, 4);
        assert_eq!(result.len, 0);
    }

    #[test]
    fn test_no_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0x100;
        // carry is set, so it should not jump
        gb.cpu.f = CARRY_FLAG_MASK;
        gb.write16(gb.cpu.pc + 1, 0x200);

        let mut instr = Jp::new(IT::JumpToImm16(
            JC::NotCarry(gb.cpu.not_carry()),
            gb.read16(gb.cpu.pc + 1),
        ));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x100);
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len, 3);
    }
}
