use crate::core::instructions::InstructionEffect;

/// copy the value stored in a src register to dst register
pub fn ld_r8_r8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 1,
        len: 1,
        flags: None,
    }
}

/// copy immediate value into dst register
pub fn ld_r8_n8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 2,
        len: 2,
        flags: None,
    }
}

/// copy 16 bits immediate value into dst pair of registers
pub fn ld_r16_n16(dst: &mut u16, src: u16) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 3,
        len: 3,
        flags: None,
    }
}

/// copy the value stored in a src register to the memory addressed by hl register pair
pub fn ld_hl_r8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 2,
        len: 1,
        flags: None,
    }
}

/// copy immediate value into the memory addressed by hl register pair
pub fn ld_hl_n8(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 3,
        len: 2,
        flags: None,
    }
}

/// copy the value pointed by hl into a dst register
pub fn ld_r8_hl(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 2,
        len: 1,
        flags: None,
    }
}

/// copy the value in src register a to the memory pointed by the dst 16 bits pair of registers
pub fn ld_r8_a(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 2,
        len: 1,
        flags: None,
    }
}

/// copy the src value in register a to the memory pointed by the 16 bits immediate value
pub fn ld_n16_a(dst: &mut u8, src: u8) -> InstructionEffect {
    *dst = src;

    InstructionEffect {
        cycles: 4,
        len: 3,
        flags: None,
    }
}
