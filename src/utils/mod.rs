/// Convert little-endian u8 pair to u16
pub fn to_u16(high: u8, low: u8) -> u16 { ((high as u16) << 8) | (low as u16) }

/// Convert u16 to little-endian u8 pair, returning (high, low)
pub fn to_u8(word: u16) -> (u8, u8) { (high(word), low(word)) }

/// Get little-endian high byte from u16
pub fn high(word: u16) -> u8 { (word >> 8) as u8 }

/// Get little-endian low byte from u16
pub fn low(word: u16) -> u8 { word as u8 }

/// take a u16 expressed as a u8 tuple (high, low) and apply a u16 -> u16 closure to it
// pub fn with_u16<F>(word: (u8, u8), f: F) -> (u8, u8)
// where
//     F: FnOnce(u16) -> u16,
// {
//     to_u8(f(to_u16(word.0, word.1)))
// }
pub fn with_u16<F>(high: &mut u8, low: &mut u8, f: F) -> (u8, u8)
where
    F: FnOnce(u16) -> u16,
{
    let result = f(to_u16(*high, *low));
    let (new_high, new_low) = to_u8(result);
    *high = new_high;
    *low = new_low;
    (new_high, new_low)
}

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

    #[test]
    fn test_with_u16() {
        let mut high = 0x12;
        let mut low = 0x34;
        let (new_high, new_low) = with_u16(&mut high, &mut low, |v| v.wrapping_add(1));
        assert_eq!((new_high, new_low), (high, low));
        assert_eq!((high, low), (0x12, 0x35));

        // test non parameter closure
        let src = 0x6767;
        let (new_high, new_low) = with_u16(&mut high, &mut low, |_| src);
        assert_eq!((new_high, new_low), (high, low));
        assert_eq!((high, low), (0x67, 0x67));
    }
}
