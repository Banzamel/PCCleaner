//! Backups made before anything the Recycle Bin cannot undo: `.reg` files for registry keys and
//! values (double-click restores them) and `.xml` files for Task Scheduler tasks (Import Task).

use std::path::Path;

use crate::system::registry::{self, ValueKind};
use crate::system::{command, tasks};
use crate::text::Text;

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The .reg text of ONE value, so a double-click restores only it, not the whole Run key.
/// `None` for unusual types (binary, numbers) - those are exported with the key by reg.exe.
pub fn value_reg_text(key: &str, name: &str, kind: ValueKind, value: &str) -> Option<String> {
    let entry = match kind {
        ValueKind::String => format!("\"{}\"=\"{}\"", escape(name), escape(value)),
        ValueKind::ExpandString => {
            let hex: Vec<String> = value.encode_utf16().chain([0]).flat_map(|c| c.to_le_bytes()).map(|b| format!("{b:02x}")).collect();
            format!("\"{}\"=hex(2):{}", escape(name), hex.join(","))
        }
        ValueKind::Other => return None,
    };
    Some(format!("Windows Registry Editor Version 5.00\r\n\r\n[{key}]\r\n{entry}\r\n"))
}

/// UTF-16 with BOM, the encoding regedit and Task Scheduler expect.
pub fn write_utf16(file: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let bytes: Vec<u8> = [0xFF, 0xFE].into_iter().chain(text.encode_utf16().flat_map(|c| c.to_le_bytes())).collect();
    std::fs::write(file, bytes)
}

/// Exports a whole key with reg.exe. Without a backup the key is not deleted.
pub fn export_key(key: &str, file: &Path) -> Result<(), Text> {
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let r = command::run(&command::system32("reg.exe"), &["export", key, &file.to_string_lossy(), "/y"]);
    if r.ok() && file.exists() { Ok(()) } else { Err(Text::new("backup failed - key kept")) }
}

/// Backs up one registry value; without a backup the value is not deleted.
pub fn export_value(key: &str, name: &str, file: &Path) -> Result<(), Text> {
    let failed = || Text::new("backup failed - entry kept");
    let Some((value, kind)) = registry::read_raw_string(key, name).or_else(|| registry::read_binary(key, name).map(|_| (String::new(), ValueKind::Other))) else {
        return Err(failed());
    };
    match value_reg_text(key, name, kind, &value) {
        Some(text) => write_utf16(file, &text).map_err(|_| failed())?,
        None => export_key(key, file).map_err(|_| failed())?,
    }
    if file.exists() { Ok(()) } else { Err(failed()) }
}

/// Saves a task's XML (Task Scheduler > Import Task restores it).
pub fn export_task(path: &str, file: &Path) -> Result<(), Text> {
    let failed = || Text::new("backup failed - task kept");
    let xml = tasks::export_xml(path).map_err(|_| failed())?;
    if xml.is_empty() {
        return Err(failed());
    }
    write_utf16(file, &xml).map_err(|_| failed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_value() {
        let t = value_reg_text("HKEY_CURRENT_USER\\Run", "App \"x\"", ValueKind::String, "\"C:\\A\\a.exe\" -m").unwrap();
        assert!(t.contains("[HKEY_CURRENT_USER\\Run]\r\n\"App \\\"x\\\"\"=\"\\\"C:\\\\A\\\\a.exe\\\" -m\""));
    }

    #[test]
    fn expand_string_value_is_hex() {
        let t = value_reg_text("HKLM\\K", "A", ValueKind::ExpandString, "%T%").unwrap();
        assert!(t.ends_with("\"A\"=hex(2):25,00,54,00,25,00,00,00\r\n"));
        assert_eq!(value_reg_text("K", "A", ValueKind::Other, ""), None);
    }
}
