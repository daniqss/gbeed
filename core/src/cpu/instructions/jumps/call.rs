use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult, JumpCondition},
    },
    prelude::*,
};

/// call given address
/// pushes the next instruction address on the stack, and then jumps to it
pub struct Call {
    jc: JumpCondition,
    n16: u16,
}

impl Call {
    pub fn new(jc: JumpCondition, n16: u16) -> Box<Self> { Box::new(Self { jc, n16 }) }
}

impl Instruction for Call {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        if !self.jc.should_jump() {
            return Ok(InstructionEffect::new(self.info(), Flags::none()));
        }

        let return_addr = gb.cpu.pc.wrapping_add(3);

        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb.write(sp, utils::high(return_addr));

        sp = sp.wrapping_sub(1);
        gb.write(sp, utils::low(return_addr));
        gb.cpu.sp = sp;

        gb.cpu.pc = self.n16;

        Ok(InstructionEffect::with_jump(self.info(), Flags::none()))
    }

    fn info(&self) -> (u8, u8) {
        if !self.jc.should_jump() {
            (3, 3)
        } else {
            (6, 3)
        }
    }
    fn disassembly(&self) -> String { format!("call {}${:04X}", self.jc, self.n16) }
}
