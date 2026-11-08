//! The StartupApproved values Task Manager writes: 12 bytes, an odd first byte means "turned off",
//! followed by the time it was turned off.

/// Missing value or an even first byte means the entry is on.
pub fn is_enabled(value: Option<&[u8]>) -> bool {
    !matches!(value, Some(b) if !b.is_empty() && b[0] & 1 == 1)
}

/// FILETIME (100 ns since 1601) of a Unix time in seconds.
pub fn filetime(unix_secs: u64) -> u64 {
    (unix_secs + 11_644_473_600) * 10_000_000
}

/// The value to write: 02 00.. for on, 03 00 00 00 + FILETIME for off.
pub fn value(enable: bool, now_unix: u64) -> [u8; 12] {
    let mut b = [0u8; 12];
    if enable {
        b[0] = 2;
    } else {
        b[0] = 3;
        b[4..12].copy_from_slice(&filetime(now_unix).to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states() {
        assert!(is_enabled(None));
        assert!(is_enabled(Some(&[2, 0, 0, 0])));
        assert!(!is_enabled(Some(&[3, 0, 0, 0])));
        assert!(is_enabled(Some(&value(true, 0))));
        assert!(!is_enabled(Some(&value(false, 1_700_000_000))));
    }

    #[test]
    fn filetime_epoch() {
        assert_eq!(filetime(0), 116_444_736_000_000_000);
    }
}
