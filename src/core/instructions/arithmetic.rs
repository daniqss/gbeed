use crate::core::{
    cpu::{CARRY_FLAG_MASK, check_carry, check_half_carry, check_zero},
    instructions::InstructionEffect,
};

/// add the value in register plus the carry flag to A register
pub fn adc_a_r8(r8: u8, f: u8, a: &mut u8) -> InstructionEffect {
    let mut result = *a + r8;
    result += if (f & CARRY_FLAG_MASK) != 0 { 1 } else { 0 };

    InstructionEffect::new(
        1,
        1,
        Some(check_zero(result) | check_carry(result, *a) | check_half_carry(result, *a)),
    )
}
