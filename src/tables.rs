//! (brightness, warmth) -> LM3630A register mapping

use crate::tables_data::{BA, BB, CA, CB};

/// Brightness and warmth levels are 0..=[`MAX`]
pub const MAX: u8 = 32;

/// Android brightness-slider value (0..=255) -> frontlight level (0..=32)
static LEVELS: [u8; 256] = [
    0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5,
    5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9,
    10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13,
    13, 13, 13, 14, 14, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16, 16, 16, 17, 17,
    17, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20, 20, 20, 20, 20, 20,
    20, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 23, 23, 23, 24, 24, 24, 24,
    24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 27, 28,
    28, 28, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31,
    31, 31, 32, 32, 32, 32,
];

/// Frontlight level for a slider value
#[must_use]
pub fn level(slider: u8) -> u8 {
    LEVELS.get(usize::from(slider)).copied().unwrap_or(0)
}

/// One table cell, `0` if indexes somehow out of range
fn cell(table: &[[u8; 33]; 33], warmth: usize, br: usize) -> u8 {
    table.get(warmth).and_then(|row| row.get(br)).copied().unwrap_or(0)
}

/// The chip's four state registers 0x03..=0x06
/// 0x03 bank A code, 0x04 bank B code, 0x05 bank A current, 0x06 bank B current
#[must_use]
pub fn codes(br: u8, warmth: u8) -> [u8; 4] {
    let warmth = usize::from(warmth.min(MAX));
    let br = usize::from(br.min(MAX));
    [
        cell(&BA, warmth, br),
        cell(&BB, warmth, br),
        cell(&CA, warmth, br),
        cell(&CB, warmth, br),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Register values observed on the device while the stock driver mixed
    #[test]
    fn matches_device_dumps() {
        for (br, warmth, want) in [
            (8u8, 0u8, [0x35, 0x00, 0x1f, 0x00]),
            (8, 16, [0x1c, 0x1c, 0x1f, 0x1f]),
            (8, 32, [0x00, 0x35, 0x00, 0x1f]),
            (16, 0, [0x78, 0x00, 0x1f, 0x00]),
            (16, 16, [0x5f, 0x5f, 0x1f, 0x1f]),
            (16, 32, [0x00, 0x78, 0x00, 0x1f]),
            (24, 0, [0xbc, 0x00, 0x1f, 0x00]),
            (24, 16, [0xa3, 0xa3, 0x1f, 0x1f]),
            (24, 32, [0x00, 0xbc, 0x00, 0x1f]),
            (32, 0, [0xff, 0x00, 0x1f, 0x00]),
            (32, 16, [0xe6, 0xe6, 0x1f, 0x1f]),
            (32, 32, [0x00, 0xff, 0x00, 0x1f]),
        ] {
            assert_eq!(codes(br, warmth), want, "br={br} warmth={warmth}");
        }
    }

    #[test]
    fn fully_warm_is_bank_b_only() {
        assert_eq!(codes(32, 32), [0x00, 0xff, 0x00, 0x1f]);
    }

    #[test]
    fn off_is_all_zero() {
        assert_eq!(codes(0, 0), [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn slider_endpoints() {
        assert_eq!(level(0), 0);
        assert_eq!(level(10), 1);
        assert_eq!(level(255), 32);
    }

    #[test]
    fn out_of_range_clamps() {
        assert_eq!(codes(255, 255), codes(32, 32));
    }
}
