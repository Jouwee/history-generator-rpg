pub(crate) fn bitmask_get(bits: u8, mask: u8) -> bool {
    return bits & mask > 0
}

pub(crate) fn bitmask_set(bits: u8, mask: u8) -> u8 {
    return bits | mask
}

pub(crate) fn bitmask_unset(bits: u8, mask: u8) -> u8 {
    return bits ^ mask
}


#[cfg(test)]
mod tests_bitmask {
    use super::*;

    #[test]
    fn test_u8() {

        const MASK_A: u8 = 0b0000_0001;
        const MASK_B: u8 = 0b0000_0010;
        const MASK_C: u8 = 0b0000_0100;
        const MASK_D: u8 = 0b0000_1000;

        let mut bits = 0b0000_0101;

        assert_eq!(bitmask_get(bits, MASK_A), true);
        assert_eq!(bitmask_get(bits, MASK_B), false);
        assert_eq!(bitmask_get(bits, MASK_C), true);
        assert_eq!(bitmask_get(bits, MASK_D), false);

        bits = bitmask_unset(bits, MASK_A);
        assert_eq!(bits, 0b0000_0100);

        bits = bitmask_set(bits, MASK_B);
        assert_eq!(bits, 0b0000_0110);

        assert_eq!(bitmask_get(bits, MASK_A), false);
        assert_eq!(bitmask_get(bits, MASK_B), true);
        assert_eq!(bitmask_get(bits, MASK_C), true);
        assert_eq!(bitmask_get(bits, MASK_D), false);

    }


}
