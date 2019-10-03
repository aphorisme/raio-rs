/// Combines the higher nibble and the lower nibble of two `u8`.
/// # Example
/// ```
/// # use raio::packing::ll::*;
/// assert_eq!(combine_nibble(0xC0, 1), 0xC1);
/// assert_eq!(combine_nibble(0xF2, 9), 0xF9);
/// assert_eq!(combine_nibble(0xB1, 0xFF), 0xBF);
/// ```
pub fn combine_nibble(high: u8, low: u8) -> u8 {
    (high & 0xF0) | (low & 0x0F)
}

/// Computes an `u8` with the first nibble set to `0000`.
/// # Example
/// ```
/// # use raio::packing::ll::*;
/// assert_eq!(low_nibble(0xC2), 2);
/// assert_eq!(low_nibble(0xAF), 0x0F);
/// ```
pub fn low_nibble(nibble: u8) -> u8 {
    nibble & 0x0F
}

/// Computes an `u8` with the last nibble set to `0000`.
/// # Example
/// ```
/// # use raio::packing::ll::*;
/// assert_eq!(high_nibble(0xF3), 0xF0);
/// assert_eq!(high_nibble(0x9D), 0x90);
/// ```
pub fn high_nibble(nibble: u8) -> u8 {
    nibble & 0xF0
}
