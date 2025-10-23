use crate::core::{
    cpu::{CARRY_FLAG_MASK, check_carry, check_half_carry, check_zero},
    instructions::InstructionEffect,
};

/// add the value in register plus the carry flag to A register
/// probably should I use wrapping_add here?
pub fn adc_a_r8(r8: u8, f: u8, a: &mut u8) -> InstructionEffect {
    let mut result = *a + r8;
    result += if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 };

    InstructionEffect::new(
        1,
        1,
        Some(check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a)),
    )
}

/// add the value pointed by HL pair of registers plus de carry to A register
pub fn adc_a_hl(value: u8, f: u8, a: &mut u8) -> InstructionEffect {
    let mut result = *a + value;
    result += if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 };

    InstructionEffect::new(
        2,
        1,
        Some(check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a)),
    )
}

/// add immediate value n8 plus the carry flag to A register
pub fn adc_a_n8(n8: u8, f: u8, a: &mut u8) -> InstructionEffect {
    let mut result = *a + n8;
    result += if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 };

    InstructionEffect::new(
        2,
        2,
        Some(check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a)),
    )
}
