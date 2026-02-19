use crate::{
    Dmg,
    core::cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition},
    },
};

pub struct JpToImm16 {
    pub jc: JumpCondition,
    pub addr: u16,
}

impl JpToImm16 {
    pub fn new(jc: JumpCondition, addr: u16) -> Box<Self> { Box::new(Self { jc, addr }) }
}

impl Instruction for JpToImm16 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let should_jump = self.jc.should_jump();
        if should_jump {
            gb.cpu.pc = self.addr;
            Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
        } else {
            Ok(InstructionEffect::new(self.info(), Flags::none()))
        }
    }
    fn info(&self) -> (u8, u8) { if self.jc.should_jump() { (4, 3) } else { (3, 3) } }
    fn disassembly(&self) -> String { format!("jp {}${:04X}", self.jc, self.addr) }
}

pub struct JpToHL {
    pub addr: u16,
}

impl JpToHL {
    pub fn new(addr: u16) -> Box<Self> { Box::new(Self { addr }) }
}

impl Instruction for JpToHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.pc = self.addr;
        Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (1, 1) }
    fn disassembly(&self) -> String { "jp [hl]".to_string() }
}

#[cfg(test)]
mod test {
    use crate::core::{
        Accessible16,
        cpu::{
            R16,
            flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK},
            instructions::JumpCondition as JC,
        },
    };

    use super::*;

    #[test]
    fn test_jump_to_hl() {
        let mut gb = Dmg::default();
        gb.store(R16::HL, 0x1234);

        let mut instr = JpToHL::new(gb.cpu.hl());
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x1234);
        assert_eq!(result.cycles, 1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_jump_n16() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0xC000;
        gb.store(gb.cpu.pc + 1, 0x200);

        let mut instr = JpToImm16::new(JC::None, gb.load(gb.cpu.pc + 1));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x200);
        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0xC000;
        gb.cpu.f = ZERO_FLAG_MASK;
        gb.store(gb.cpu.pc + 1, 0x200);

        let mut instr = JpToImm16::new(JC::Zero(gb.cpu.zero()), gb.load(gb.cpu.pc + 1));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0x200);
        assert_eq!(result.cycles, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_no_jump_with_jc() {
        let mut gb = Dmg::default();
        gb.cpu.pc = 0xC000;
        // carry is set, so it should not jump
        gb.cpu.f = CARRY_FLAG_MASK;
        gb.store(gb.cpu.pc + 1, 0x200);

        let mut instr = JpToImm16::new(JC::NotCarry(gb.cpu.not_carry()), gb.load(gb.cpu.pc + 1));
        let result = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.pc, 0xC000);
        assert_eq!(result.cycles, 3);
        assert_eq!(result.len(), 3);
    }
}
