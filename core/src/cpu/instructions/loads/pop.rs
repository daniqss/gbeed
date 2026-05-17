use crate::{
    cpu::{
        R16,
        instructions::{Instruction, InstructionEffect, InstructionResult},
    },
    prelude::*,
};

/// Pop a 16 bit register from the stack.
/// Should behave like the following non-real instructions (for AF register, but its the same for the other 16 bit registers):
/// ```asm
/// ld f, [sp]
/// inc sp
/// ld a, [sp]
/// inc sp
/// ``````
#[derive(Debug, Default, Clone, Copy)]
pub struct Pop {
    dst: R16,
}

impl Pop {
    pub fn new(dst: R16) -> StaticBox<Self> { StaticBox::new(Self { dst }) }
}
impl Instruction for Pop {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let src = gb.load(gb.cpu.sp);

        // pop from stack to register
        gb.store(self.dst, src);

        // increment stack pointer by 2, one for each byte popped
        gb.cpu.sp = gb.cpu.sp.wrapping_add(2);

        Ok(InstructionEffect::new(self.info(), None))
    }
    fn info(&self) -> (u8, u8) { (3, 1) }
    fn disassembly(&self) -> String { format!("pop {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cpu::flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK};

    #[test]
    fn test_pop_to_af() {
        let mut gb = Dmg::default();
        gb.cpu.set_f(0);
        gb.cpu.a = 0;
        gb.cpu.sp = 0xC000;

        let sp = gb.cpu.sp;
        gb.write(sp, ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        gb.write(sp + 1, 1);

        let mut instr = Pop::new(R16::AF);
        let effect = instr.exec(&mut gb).unwrap();

        assert_eq!(gb.cpu.a, 1);
        assert!(gb.cpu.zero());
        assert!(!gb.cpu.subtraction());
        assert!(!gb.cpu.half_carry());
        assert!(gb.cpu.carry());
        assert_eq!(gb.cpu.sp, 0xC002);
        assert_eq!(effect.cycles, 3);
        assert_eq!(effect.len(), 1);
        assert!(effect.flags.is_none());
    }
}
