//! Registry access by full path (`HKEY_LOCAL_MACHINE\SOFTWARE\...`), the form the old program used.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    String,
    ExpandString,
    Other,
}

#[cfg(windows)]
mod imp {
    use super::ValueKind;
    use winreg::enums::*;
    use winreg::{RegKey, RegValue};

    fn split(path: &str) -> Option<(RegKey, &str)> {
        let (root, sub) = path.split_once('\\').unwrap_or((path, ""));
        let hive = match root.to_ascii_uppercase().as_str() {
            "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
            "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
            "HKEY_USERS" | "HKU" => HKEY_USERS,
            "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
            _ => return None,
        };
        Some((RegKey::predef(hive), sub))
    }

    pub fn open(path: &str, access: u32) -> Option<RegKey> {
        let (root, sub) = split(path)?;
        if sub.is_empty() {
            return Some(root);
        }
        root.open_subkey_with_flags(sub, access).ok()
    }

    pub fn raw(path: &str, name: &str) -> Option<RegValue<'static>> {
        open(path, KEY_READ)?.get_raw_value(name).ok()
    }

    pub fn kind(v: &RegValue) -> ValueKind {
        match v.vtype {
            REG_SZ => ValueKind::String,
            REG_EXPAND_SZ => ValueKind::ExpandString,
            _ => ValueKind::Other,
        }
    }

    pub fn create(path: &str) -> std::io::Result<RegKey> {
        let (root, sub) = split(path).ok_or_else(|| std::io::Error::other("unknown registry hive"))?;
        root.create_subkey(sub).map(|(k, _)| k)
    }

    pub fn delete_tree(path: &str) -> std::io::Result<()> {
        let (root, sub) = split(path).ok_or_else(|| std::io::Error::other("unknown registry hive"))?;
        if sub.is_empty() {
            return Err(std::io::Error::other("refusing to delete a registry hive"));
        }
        root.delete_subkey_all(sub)
    }
}

/// String value; REG_EXPAND_SZ is expanded like PowerShell's Get-ItemProperty does.
#[cfg(windows)]
pub fn read_string(path: &str, name: &str) -> Option<String> {
    use winreg::types::FromRegValue;
    let v = imp::raw(path, name)?;
    let s = String::from_reg_value(&v).ok()?;
    Some(if imp::kind(&v) == ValueKind::ExpandString { expand_env(&s) } else { s })
}

/// String value without expanding environment variables (for backups).
#[cfg(windows)]
pub fn read_raw_string(path: &str, name: &str) -> Option<(String, ValueKind)> {
    use winreg::types::FromRegValue;
    let v = imp::raw(path, name)?;
    Some((String::from_reg_value(&v).ok()?, imp::kind(&v)))
}

#[cfg(windows)]
pub fn read_binary(path: &str, name: &str) -> Option<Vec<u8>> {
    let v = imp::raw(path, name)?;
    (v.vtype == winreg::enums::REG_BINARY).then(|| v.bytes.into_owned())
}

#[cfg(windows)]
pub fn read_dword(path: &str, name: &str) -> Option<u32> {
    imp::open(path, winreg::enums::KEY_READ)?.get_value::<u32, _>(name).ok()
}

#[cfg(windows)]
pub fn subkeys(path: &str) -> Vec<String> {
    imp::open(path, winreg::enums::KEY_READ).map(|k| k.enum_keys().flatten().collect()).unwrap_or_default()
}

#[cfg(windows)]
pub fn value_names(path: &str) -> Vec<String> {
    imp::open(path, winreg::enums::KEY_READ).map(|k| k.enum_values().flatten().map(|(n, _)| n).collect()).unwrap_or_default()
}

#[cfg(windows)]
pub fn key_exists(path: &str) -> bool {
    imp::open(path, winreg::enums::KEY_READ).is_some()
}

#[cfg(windows)]
pub fn set_binary(path: &str, name: &str, bytes: &[u8]) -> std::io::Result<()> {
    let v = winreg::RegValue { bytes: std::borrow::Cow::Owned(bytes.to_vec()), vtype: winreg::enums::REG_BINARY };
    imp::create(path)?.set_raw_value(name, &v)
}

#[cfg(windows)]
pub fn set_dword(path: &str, name: &str, value: u32) -> std::io::Result<()> {
    imp::open(path, winreg::enums::KEY_SET_VALUE).ok_or_else(|| std::io::Error::other("key not found"))?.set_value(name, &value)
}

#[cfg(windows)]
pub fn delete_value(path: &str, name: &str) -> std::io::Result<()> {
    imp::open(path, winreg::enums::KEY_SET_VALUE).ok_or_else(|| std::io::Error::other("key not found"))?.delete_value(name)
}

#[cfg(windows)]
pub fn delete_tree(path: &str) -> std::io::Result<()> {
    imp::delete_tree(path)
}

#[cfg(windows)]
pub fn expand_env(s: &str) -> String {
    use windows::core::PCWSTR;
    use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
    let src = super::wide(s);
    let mut buf = vec![0u16; 32_768];
    // SAFETY: both buffers are valid for the given lengths.
    let n = unsafe { ExpandEnvironmentStringsW(PCWSTR(src.as_ptr()), Some(&mut buf)) } as usize;
    if n == 0 || n > buf.len() {
        return s.to_string();
    }
    String::from_utf16_lossy(&buf[..n.saturating_sub(1)])
}

#[cfg(not(windows))]
mod fallback {
    use super::ValueKind;
    pub fn read_string(_: &str, _: &str) -> Option<String> { None }
    pub fn read_raw_string(_: &str, _: &str) -> Option<(String, ValueKind)> { None }
    pub fn read_binary(_: &str, _: &str) -> Option<Vec<u8>> { None }
    pub fn read_dword(_: &str, _: &str) -> Option<u32> { None }
    pub fn subkeys(_: &str) -> Vec<String> { Vec::new() }
    pub fn value_names(_: &str) -> Vec<String> { Vec::new() }
    pub fn key_exists(_: &str) -> bool { false }
    pub fn set_binary(_: &str, _: &str, _: &[u8]) -> std::io::Result<()> { Err(std::io::Error::other("unsupported")) }
    pub fn set_dword(_: &str, _: &str, _: u32) -> std::io::Result<()> { Err(std::io::Error::other("unsupported")) }
    pub fn delete_value(_: &str, _: &str) -> std::io::Result<()> { Err(std::io::Error::other("unsupported")) }
    pub fn delete_tree(_: &str) -> std::io::Result<()> { Err(std::io::Error::other("unsupported")) }
    pub fn expand_env(s: &str) -> String { s.to_string() }
}
#[cfg(not(windows))]
pub use fallback::*;
