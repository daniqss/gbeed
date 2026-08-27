pub(crate) mod macros;

/// Convert little-endian u8 pair to u16
#[inline(always)]
pub fn to_u16(low: u8, high: u8) -> u16 { ((high as u16) << 8) | (low as u16) }

/// Get little-endian high byte from u16
#[inline(always)]
pub fn high(word: u16) -> u8 { (word >> 8) as u8 }

/// Get little-endian low byte from u16
#[inline(always)]
pub fn low(word: u16) -> u8 { word as u8 }

#[inline(always)]
pub fn from_u16(low_byte: &mut u8, high_byte: &mut u8, word: u16) {
    *low_byte = low(word);
    *high_byte = high(word);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_u16() {
        assert_eq!(to_u16(0x34, 0x12), 0x1234);
        assert_eq!(to_u16(0x00, 0x00), 0x0000);
        assert_eq!(to_u16(0xFF, 0xFF), 0xFFFF);
    }
}
