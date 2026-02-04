use crate::{
    Dmg,
    core::{
        Accessible16,
        cpu::{
            R8, R16,
            flags::{Flags, check_borrow_hc, check_zero},
            instructions::{Instruction, InstructionEffect, InstructionResult},
        },
        memory::Accessible,
    },
};

#[inline(always)]
fn dec_u8_flags(old: u8, result: u8) -> Flags {
    Flags {
        z: Some(check_zero(result)),
        n: Some(true),
        h: Some(check_borrow_hc(old, 1)),
        c: None,
    }
}

pub struct DecR8 {
    dst: R8,
}

impl Instruction for DecR8 {
    type Args = R8;

    fn new(args: Self::Args) -> Box<Self> { Box::new(DecR8 { dst: args }) }

    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r8 = gb.read(self.dst);
        let result = r8.wrapping_sub(1);
        gb.write(self.dst, result);

        Ok(InstructionEffect::new(1, 1, dec_u8_flags(r8, result)))
    }

    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

pub struct DecPointedByHL;

impl Instruction for DecPointedByHL {
    type Args = ();

    fn new(_: Self::Args) -> Box<Self> { Box::new(DecPointedByHL) }

    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let n8 = gb.read(gb.cpu.hl());
        let result = n8.wrapping_sub(1);
        gb.write(gb.cpu.hl(), result);

        Ok(InstructionEffect::new(3, 1, dec_u8_flags(n8, result)))
    }

    fn disassembly(&self) -> String { format!("dec [hl]") }
}

pub struct DecR16 {
    dst: R16,
}

impl Instruction for DecR16 {
    type Args = R16;

    fn new(args: Self::Args) -> Box<Self> { Box::new(DecR16 { dst: args }) }

    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let r16 = gb.load(self.dst);
        let result = r16.wrapping_sub(1);
        gb.store(self.dst, result);

        Ok(InstructionEffect::new(2, 1, Flags::none()))
    }

    fn disassembly(&self) -> String { format!("dec {}", self.dst) }
}

pub struct DecStackPointer;

impl Instruction for DecStackPointer {
    type Args = ();

    fn new(_: Self::Args) -> Box<Self> { Box::new(DecStackPointer) }
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        gb.cpu.sp = gb.cpu.sp.wrapping_sub(1);

        Ok(InstructionEffect::new(2, 1, Flags::none()))
    }

    fn disassembly(&self) -> String { format!("dec sp") }
}
