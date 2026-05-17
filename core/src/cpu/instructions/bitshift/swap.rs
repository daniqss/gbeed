use crate::{
    cpu::{
        R8,
        flags::{
            CARRY_FLAG_MASK, HALF_CARRY_FLAG_MASK, LazyFlags, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK,
            check_zero,
        },
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct SwapR8 {
    dst: R8,
}

impl SwapR8 {
    pub fn new(dst: R8) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}

impl Instruction for SwapR8 {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.rotate_right(4);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SwapFlags::new(result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (2, 2) }
    fn disassembly(&self) -> String { format!("swap {}", self.dst) }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SwapPointedByHL;
impl SwapPointedByHL {
    pub fn new() -> StaticBox<Self> { StaticBox::new(Self) }
}
impl Instruction for SwapPointedByHL {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.rotate_right(4);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(
            self.info(),
            Some(SwapFlags::new(result).into()),
        ))
    }
    fn info(&self) -> (u8, u8) { (4, 2) }
    fn disassembly(&self) -> String { "swap [hl]".to_string() }
}

#[derive(Debug, Default, Clone, Copy)]
struct SwapFlags {
    result: u8,
}

impl SwapFlags {
    fn new(result: u8) -> StaticBox<Self> { StaticBox::new(Self { result }) }
}

impl LazyFlags for SwapFlags {
    fn updated_flags(&self) -> u8 {
        ZERO_FLAG_MASK | SUBTRACTION_FLAG_MASK | HALF_CARRY_FLAG_MASK | CARRY_FLAG_MASK
    }

    fn zero(&self) -> bool { check_zero(self.result) }
    fn subtraction(&self) -> bool { false }
    fn half_carry(&self) -> bool { false }
    fn carry(&self) -> bool { false }
}
