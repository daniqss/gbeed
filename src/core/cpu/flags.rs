/// # Flag masks
/// The F register use its 4 most significant bits to store information about the result of the previous operation
pub const ZERO_FLAG_MASK: u8 = 0b1000_0000;
pub const SUBTRACTION_FLAG_MASK: u8 = 0b0100_0000;
pub const HALF_CARRY_FLAG_MASK: u8 = 0b0010_0000;
pub const CARRY_FLAG_MASK: u8 = 0b0001_0000;

pub fn check_zero(result: u8) -> u8 { if result == 0 { ZERO_FLAG_MASK } else { 0 } }
pub fn check_half_carry(result: u8, old: u8) -> u8 {
    if (result & 0x0F) < (old & 0x0F) {
        HALF_CARRY_FLAG_MASK
    } else {
        0
    }
}
pub fn check_carry(result: u8, old: u8) -> u8 { if result < old { CARRY_FLAG_MASK } else { 0 } }

// needs refactor by actually using instructions that return different flags
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::low;

    fn simulate_add_sp_e8(sp: u16, e8: i8) -> (u16, u8) {
        let old_sp = sp;
        let result = old_sp.wrapping_add(e8 as u16);

        let flags = check_carry(low(result), low(old_sp)) | check_half_carry(low(result), low(old_sp));

        (result, flags)
    }

    #[test]
    fn test_add_sp_no_carries() {
        let (new_sp, flags) = simulate_add_sp_e8(0x1000, 2);

        assert_eq!(new_sp, 0x1002);
        assert_eq!(flags, 0);
    }

    #[test]
    fn test_add_sp_half_carry_only() {
        let (new_sp, flags) = simulate_add_sp_e8(0x100F, 1);

        assert_eq!(new_sp, 0x1010);
        assert_eq!(flags, HALF_CARRY_FLAG_MASK);
    }

    #[test]
    fn test_add_sp_full_carry_only() {
        let (new_sp, flags) = simulate_add_sp_e8(0x10F0, 32);

        assert_eq!(new_sp, 0x1110);
        assert_eq!(flags, CARRY_FLAG_MASK);
    }

    #[test]
    fn test_add_sp_both_carries() {
        let (new_sp, flags) = simulate_add_sp_e8(0x10FF, 1);

        assert_eq!(new_sp, 0x1100);
        let expected = CARRY_FLAG_MASK | HALF_CARRY_FLAG_MASK;
        assert_eq!(flags, expected);
    }

    #[test]
    fn test_add_sp_flags_z_and_n_are_always_zero() {
        let (new_sp, flags) = simulate_add_sp_e8(0x10FF, 1);

        assert_eq!(low(new_sp), 0x00);
        assert_eq!((flags & ZERO_FLAG_MASK), 0);
        assert_eq!((flags & SUBTRACTION_FLAG_MASK), 0);

        let expected_hc = CARRY_FLAG_MASK | HALF_CARRY_FLAG_MASK;
        assert_eq!(flags, expected_hc);
    }

    #[test]
    fn test_add_sp_negative_e8_carries() {
        let (new_sp, flags) = simulate_add_sp_e8(0x100F, -1);

        assert_eq!(new_sp, 0x100E);
        let expected = CARRY_FLAG_MASK | HALF_CARRY_FLAG_MASK;
        assert_eq!(flags, expected);

        assert_eq!((flags & ZERO_FLAG_MASK), 0);
        assert_eq!((flags & SUBTRACTION_FLAG_MASK), 0);
    }
}
