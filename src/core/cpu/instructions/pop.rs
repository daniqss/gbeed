use std::fmt::Write;

use crate::{
    core::cpu::{
        R16,
        flags::{CARRY_FLAG_MASK, Flags, HALF_CARRY_FLAG_MASK, SUBTRACTION_FLAG_MASK, ZERO_FLAG_MASK},
        instructions::{
            Instruction, InstructionDestination as ID, InstructionEffect, InstructionError, InstructionResult,
        },
    },
    utils::to_u8,
};

/// Pop a 16 bit register from the stack.
/// Should behave like the following non-real instructions (for AF register, but its the same for the other 16 bit registers):
/// ```asm
/// ld f, [sp]
/// inc sp
/// ld a, [sp]
/// inc sp
/// ``````
pub struct Pop<'a> {
    dst: ID<'a>,
    src: u16,
    sp: &'a mut u16,
}

impl<'a> Pop<'a> {
    pub fn new(dst: ID<'a>, src: u16, sp: &'a mut u16) -> Box<Self> { Box::new(Self { dst, src, sp }) }
}
impl<'a> Instruction<'a> for Pop<'a> {
    fn exec(&mut self) -> InstructionResult {
        let (low, high) = to_u8(self.src);

        let (dst, flags) = match &mut self.dst {
            ID::Reg16(dst, R16::AF) => (
                dst,
                // set flags according to the bits that are going to pop into F
                Flags {
                    z: Some(low & ZERO_FLAG_MASK != 0),
                    n: Some(low & SUBTRACTION_FLAG_MASK != 0),
                    h: Some(low & HALF_CARRY_FLAG_MASK != 0),
                    c: Some(low & CARRY_FLAG_MASK != 0),
                },
            ),
            ID::Reg16(dst, _) => (dst, Flags::none()),
            _ => return Err(InstructionError::MalformedInstruction),
        };

        // pop [sp] to low register, such as F in AF
        *dst.0 = low;

        // pop [sp+1] to high register, such as A in AF
        *dst.1 = high;

        // increment stack pointer by 2, one for each byte popped
        *self.sp = self.sp.wrapping_add(2);

        Ok(InstructionEffect::new(3, 1, flags))
    }

    fn disassembly(&self, w: &mut dyn Write) -> Result<(), std::fmt::Error> { write!(w, "pop {}", self.dst) }
}

#[cfg(test)]
mod tests {
    use crate::core::memory::Memory;

    use super::*;

    #[test]
    fn test_pop_to_af() {
        let mut f = 0;
        let mut a = 0;
        let mut sp = 0xFF00;
        let bus = Memory::new(None, None, None);

        bus.borrow_mut()[sp] = ZERO_FLAG_MASK | CARRY_FLAG_MASK;
        bus.borrow_mut()[sp + 1] = 1;

        let mut instr = Pop::new(
            ID::Reg16((&mut f, &mut a), R16::AF),
            bus.borrow().read_word(sp),
            &mut sp,
        );
        let effect = instr.exec().unwrap();

        assert_eq!(a, 1);
        assert_eq!(f, ZERO_FLAG_MASK | CARRY_FLAG_MASK);
        assert_eq!(sp, 0xFF02);
        assert_eq!(effect.cycles, 3);
        assert_eq!(effect.len, 1);
        assert_eq!(
            effect.flags,
            Flags {
                z: Some(true),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
        assert_eq!(
            effect.flags,
            Flags {
                z: Some(f & ZERO_FLAG_MASK != 0),
                n: Some(f & SUBTRACTION_FLAG_MASK != 0),
                h: Some(f & HALF_CARRY_FLAG_MASK != 0),
                c: Some(f & CARRY_FLAG_MASK != 0),
            }
        );
    }
}
