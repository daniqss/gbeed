use crate::{
    cpu::{
        flags::Flags,
        instructions::{Instruction, InstructionEffect, InstructionResult},
        R16,
    },
    prelude::*,
};

/// Push a 16 bit register onto the stack. It is roughly equivalent to the following imaginary instructions:
/// ```asm
/// dec sp
/// ld [sp], a
/// dec sp
/// ld [sp], F.Z << 7 | F.N << 6 | F.H << 5 | F.C << 4
/// ```
/// for other 16 bits registers, it'll look the same, but without the flags logic, like this:
/// ```asm
///  dec sp
/// ld [sp], b  ; B, D or H
/// dec sp
/// ld [sp], c   ; C, E or L
pub struct Push {
    src: R16,
}

impl Push {
    pub fn new(src: R16) -> Box<Self> { Box::new(Self { src }) }
}

impl Instruction for Push {
    fn exec(&mut self, gb: &mut Dmg) -> InstructionResult {
        let src = match self.src {
            R16::AF => (gb.cpu.f & 0b1111_0000, gb.cpu.a),
            _ => utils::to_u8(gb.load(self.src)),
        };

        // let src = to_u16(src.0, src.1);
        let mut sp = gb.cpu.sp.wrapping_sub(1);
        gb.write(sp, src.1);
        sp = sp.wrapping_sub(1);
        gb.write(sp, src.0);
        gb.cpu.sp = sp;
        // with u16 functions
        // gb.cpu.sp = gb.cpu.sp.wrapping_sub(2);
        // gb.store(gb.cpu.sp, src);

        Ok(InstructionEffect::new(self.info(), Flags::none()))
    }
    fn info(&self) -> (u8, u8) { (4, 1) }
    fn disassembly(&self) -> String { format!("push {}", self.src) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cpu::flags::{CARRY_FLAG_MASK, ZERO_FLAG_MASK};

    #[test]
    fn test_push_af() {
        let mut gb = Dmg::default();
        gb.cpu.f = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        gb.cpu.a = 1;
        gb.cpu.sp = 0xFFA0;

        let mut push = Push::new(R16::AF);

        let effect = push.exec(&mut gb).unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len(), 1);
        assert_eq!(gb.read(gb.cpu.sp), ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(gb.read(gb.cpu.sp + 1), gb.cpu.a);
        assert_eq!(gb.cpu.sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }

    #[test]
    fn test_push_bc() {
        let mut gb = Dmg::default();
        gb.cpu.c = ZERO_FLAG_MASK | CARRY_FLAG_MASK | 1;
        gb.cpu.b = 1;
        gb.cpu.sp = 0xFFA0;

        let mut push = Push::new(R16::BC);

        let effect = push.exec(&mut gb).unwrap();

        assert_eq!(effect.cycles, 4);
        assert_eq!(effect.len(), 1);
        assert_eq!(gb.read(gb.cpu.sp), gb.cpu.c);
        assert_eq!(gb.read(gb.cpu.sp + 1), gb.cpu.b);
        assert_eq!(gb.cpu.sp, 0xFFA0 - 2);
        assert_eq!(effect.flags, Flags::none());
    }
}
