use crate::core::instructions::InstructionEffect;

pub fn ld_r8_r8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 4,
        len: 1,
        flags: 0,
    }
}
