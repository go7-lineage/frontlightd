//! bionic's system properties, nothing is set on other hosts

/// bionic `PROP_VALUE_MAX` (`sys/system_properties.h`), including the NUL
#[cfg(target_os = "android")]
const PROP_VALUE_MAX: usize = 92;

#[cfg(target_os = "android")]
unsafe extern "C" {
    // Not in libc crate, bionic sys/system_properties.h
    fn __system_property_area_serial() -> u32;
}

/// The value of property `name`, `None` if unset or empty
#[cfg(target_os = "android")]
#[must_use]
pub fn get(name: &str) -> Option<String> {
    let name = std::ffi::CString::new(name).ok()?;
    let mut value = [0u8; PROP_VALUE_MAX];
    // SAFETY: `name` is NUL-terminated and `value` has PROP_VALUE_MAX bytes bionic may write
    let n = unsafe { libc::__system_property_get(name.as_ptr(), value.as_mut_ptr().cast()) };
    if n <= 0 {
        return None;
    }
    let value = std::ffi::CStr::from_bytes_until_nul(&value).ok()?;
    Some(value.to_string_lossy().into_owned())
}

/// Value of property `name`, `None` if unset or empty
#[cfg(not(target_os = "android"))]
#[must_use]
pub const fn get(_name: &str) -> Option<String> {
    None
}

/// Parses all of `s` like C's `strtol(s, &end, 0)` with `*end == '\0'`. `None` if anything is left over
#[must_use]
pub fn parse_int(s: &str) -> Option<i32> {
    let s = s.trim_start_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']);
    let (negative, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (radix, digits) = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        (16, hex)
    } else if let Some(oct) = s.strip_prefix('0').filter(|rest| !rest.is_empty()) {
        (8, oct)
    } else {
        (10, s)
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let n = i64::from_str_radix(digits, radix).ok()?;
    i32::try_from(if negative { n.checked_neg()? } else { n }).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_like_strtol_base_0() {
        for (s, want) in [
            ("20", Some(20)),
            ("0", Some(0)),
            ("-3", Some(-3)),
            ("  42", Some(42)),
            ("0x1F", Some(31)),
            ("", None),
            ("5x", None),
            ("abc", None),
        ] {
            assert_eq!(parse_int(s), want, "{s:?}");
        }
    }
}
