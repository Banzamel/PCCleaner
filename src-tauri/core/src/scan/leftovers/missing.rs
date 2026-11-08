/// Whether a path points to a program that certainly no longer exists. Only a certain absence counts:
/// an absolute path on a drive that exists (an unplugged USB drive is not a dead entry), outside the
/// Windows folder (System32/SysWOW64 redirection would give false misses) and outside network shares.
/// `exists` is injected so the rule is tested without a Windows file system.
pub fn is_missing(path: &str, windir: &str, exists: impl Fn(&str) -> bool) -> bool {
    let p = path.trim();
    let b = p.as_bytes();
    if p.is_empty() || p.starts_with("\\\\") || b.len() < 3 || !b[0].is_ascii_alphabetic() || b[1] != b':' || b[2] != b'\\' {
        return false;
    }
    let lower = p.to_lowercase();
    let win = format!("{}\\", windir.trim_end_matches('\\').to_lowercase());
    if lower.starts_with(&win) {
        return false;
    }
    if !exists(&p[..3]) || exists(p) {
        return false;
    }
    // Shortcuts of 32-bit programs may name Program Files instead of Program Files (x86) and vice versa
    for (from, to) in [("\\program files\\", "\\Program Files (x86)\\"), ("\\program files (x86)\\", "\\Program Files\\")] {
        if let Some(i) = lower.find(from) {
            let swapped = format!("{}{}{}", &p[..i], to, &p[i + from.len()..]);
            if exists(&swapped) {
                return false;
            }
        }
    }
    true
}

/// [`is_missing`] against the real file system.
pub fn is_missing_on_disk(path: &str, windir: &str) -> bool {
    is_missing(path, windir, |p| std::path::Path::new(p).exists())
}

#[cfg(test)]
mod tests {
    use super::is_missing;

    fn fs(p: &str) -> bool {
        matches!(p, "C:\\" | "C:\\Program Files (x86)\\App\\app.exe" | "C:\\Present.exe")
    }

    #[test]
    fn only_certain_absence_counts() {
        assert!(is_missing("C:\\Gone\\app.exe", "C:\\Windows", fs));
        assert!(!is_missing("C:\\Present.exe", "C:\\Windows", fs));
        assert!(!is_missing("E:\\usb\\app.exe", "C:\\Windows", fs), "drive not connected");
        assert!(!is_missing("C:\\Windows\\System32\\x.exe", "C:\\Windows", fs));
        assert!(!is_missing("\\\\server\\app.exe", "C:\\Windows", fs));
        assert!(!is_missing("app.exe", "C:\\Windows", fs));
        assert!(!is_missing("C:\\Program Files\\App\\app.exe", "C:\\Windows", fs), "32-bit redirect");
    }
}
