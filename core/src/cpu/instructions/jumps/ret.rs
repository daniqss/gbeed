use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition},
    },
    prelude::*,
};

/// return from subroutine
pub struct Ret {
    pub jc: JumpCondition,
}

impl Ret {
    pub fn new(jc: JumpCondition) -> Box<Self> { Box::new(Self { jc }) }
}

impl Instruction for Ret {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        if !self.jc.should_jump() {
            return Ok(InstructionEffect::new(self.info(), Flags::none()));
        }

        let return_addr = gb.load(gb.cpu.sp);
        gb.cpu.pc = return_addr;
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) {
        let cycles = if !self.jc.should_jump() {
            2
        } else if let JumpCondition::None = self.jc {
            4
        } else {
            5
        };
        (cycles, 1)
    }

    fn disassembly(&self) -> String {
        match self.jc {
            JumpCondition::None => "ret".to_string(),
            _ => format!("ret {}", self.jc).trim_end_matches(',').to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cpu::{flags::ZERO_FLAG_MASK, instructions::JumpCondition as JC};

    #[test]
    fn test_ret_unconditional() {
        let mut gb = Dmg::default();
        let ret_addr = 0x1234;
        let sp = 0xFFFC;
        gb.cpu.sp = sp;
        gb.store(sp, ret_addr);

        let mut instr = Ret::new(JC::None);
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, ret_addr);
        assert_eq!(gb.cpu.sp, sp.wrapping_add(2));
        assert_eq!(result.cycles, 4);
    }

    #[test]
    fn test_ret_conditional_taken() {
        let mut gb = Dmg::default();
        let ret_addr = 0x4321;
        let sp = 0xFFFC;
        gb.cpu.sp = sp;
        gb.store(sp, ret_addr);
        gb.cpu.f = ZERO_FLAG_MASK; // Set Zero flag

        let mut instr = Ret::new(JC::Zero(gb.cpu.zero()));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, ret_addr);
        assert_eq!(gb.cpu.sp, sp.wrapping_add(2));
        assert_eq!(result.cycles, 5);
    }

    #[test]
    fn test_ret_conditional_not_taken() {
        let mut gb = Dmg::default();
        let pc = 0x100;
        gb.cpu.pc = pc;
        let sp = 0xFFFC;
        gb.cpu.sp = sp;
        gb.cpu.f = 0; // Clear Zero flag

        let mut instr = Ret::new(JC::Zero(gb.cpu.zero()));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, pc);
        assert_eq!(gb.cpu.sp, sp);
        assert_eq!(result.cycles, 2);
    }
}
