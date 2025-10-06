pub fn to_u16(high: u8, low: u8) -> u16 { ((high as u16) << 8) | (low as u16) }

pub fn to_u8(word: u16) -> (u8, u8) { ((word >> 8) as u8, word as u8) }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_u16() {
        assert_eq!(to_u16(0x12, 0x34), 0x1234);
        assert_eq!(to_u16(0x00, 0x00), 0x0000);
        assert_eq!(to_u16(0xFF, 0xFF), 0xFFFF);
    }

    #[test]
    fn test_to_u8() {
        assert_eq!(to_u8(0x1234), (0x12, 0x34));
        assert_eq!(to_u8(0x0000), (0x00, 0x00));
        assert_eq!(to_u8(0xFFFF), (0xFF, 0xFF));
    }
}
