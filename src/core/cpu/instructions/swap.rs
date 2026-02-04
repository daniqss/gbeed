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

fn swap_u8_flags(result: u8) -> Flags {
    Flags {
        z: Some(result == 0),
        n: Some(false),
        h: Some(false),
        c: Some(false),
    }
}

pub struct SwapR8 {
    dst: R8,
}

impl SwapR8 {
    pub fn new(dst: R8) -> Box<Self> { Box::new(Self { dst }) }
}

impl Instruction for SwapR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = (r8 << 4) | (r8 >> 4);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(self.info(gb), swap_u8_flags(result)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("swap {}", self.dst) }
}

pub struct SwapPointedByHL;
impl SwapPointedByHL {
    pub fn new() -> Box<Self> { Box::new(Self) }
}
impl Instruction for SwapPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = (n8 << 4) | (n8 >> 4);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(self.info(gb), swap_u8_flags(result)))
    }
    fn info(&self, _: &mut Dmg) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { format!("swap [hl]") }
}
