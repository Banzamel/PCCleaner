//! Pure rules that keep vendor registry keys of installed programs out of the leftovers.
//! electron-builder (and other installers) store a per-app key under Software named exactly like the
//! app's Uninstall subkey (often a GUID) with values such as InstallLocation; deleting it breaks the
//! app's uninstaller, so such keys are never leftovers.

use std::collections::HashSet;

/// Uninstall subkey name in a comparable form: lower case, without braces and the Inno "_is1" suffix.
pub fn normalize_uninstall_name(name: &str) -> String {
    let n = name.trim().to_lowercase();
    let n = n.strip_suffix("_is1").unwrap_or(&n);
    n.trim_start_matches('{').trim_end_matches('}').to_string()
}

/// Comparable names of every Uninstall subkey (HKCU, HKLM, WOW6432Node).
pub fn uninstall_names<I: IntoIterator<Item = String>>(names: I) -> HashSet<String> {
    names.into_iter().map(|n| normalize_uninstall_name(&n)).filter(|n| !n.is_empty()).collect()
}

/// (a) The key has the name of an Uninstall entry - it belongs to an installed program.
pub fn matches_uninstall_entry(key_name: &str, uninstall: &HashSet<String>) -> bool {
    uninstall.contains(&normalize_uninstall_name(key_name))
}

/// An absolute local path in a registry value, without quotes.
fn as_path(value: &str) -> Option<&str> {
    let v = value.trim().trim_matches('"').trim();
    let b = v.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\').then_some(v)
}

/// (b) One of the key's string values (InstallLocation, Path, InstallDir...) points to an existing
/// folder or file - a program is still there.
pub fn points_to_existing(values: &[String], exists: impl Fn(&str) -> bool) -> bool {
    values.iter().filter_map(|v| as_path(v)).any(|p| exists(p))
}

/// (c) A GUID name says nothing about the program, so such a key is never selected in advance.
pub fn is_guid_name(name: &str) -> bool {
    let n = normalize_uninstall_name(name);
    let groups: Vec<&str> = n.split('-').collect();
    let lens = [8, 4, 4, 4, 12];
    groups.len() == 5 && groups.iter().zip(lens).all(|(g, l)| g.len() == l && g.chars().all(|c| c.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> HashSet<String> {
        uninstall_names(
            ["1b39727b-3969-5593-8eba-cc072b4b0548", "{6F3C2B1E-8A4D-4E7B-9C51-2D7E0F4A9B63}_is1", "Steam"].map(String::from),
        )
    }

    #[test]
    fn electron_builder_key_matches_its_uninstall_entry() {
        let n = names();
        assert!(matches_uninstall_entry("1B39727B-3969-5593-8EBA-CC072B4B0548", &n));
        assert!(matches_uninstall_entry("{1b39727b-3969-5593-8eba-cc072b4b0548}", &n));
        assert!(matches_uninstall_entry("6F3C2B1E-8A4D-4E7B-9C51-2D7E0F4A9B63", &n), "with braces and _is1 in Uninstall");
        assert!(matches_uninstall_entry("steam", &n));
        assert!(!matches_uninstall_entry("9872723e-58ea-57ac-ab07-4bdcc373d3b7", &n));
    }

    #[test]
    fn existing_install_location_keeps_the_key() {
        let exists = |p: &str| p.eq_ignore_ascii_case("P:\\Games\\LoL Helper");
        assert!(points_to_existing(&["\"P:\\Games\\LoL Helper\"".into(), "1.0".into()], exists));
        assert!(!points_to_existing(&["C:\\Gone".into(), "not a path".into(), "\\\\server\\x".into()], exists));
    }

    #[test]
    fn guid_names() {
        assert!(is_guid_name("9872723e-58ea-57ac-ab07-4bdcc373d3b7"));
        assert!(is_guid_name("{9872723E-58EA-57AC-AB07-4BDCC373D3B7}"));
        assert!(!is_guid_name("Klei"));
        assert!(!is_guid_name("1234-5678"));
    }
}
