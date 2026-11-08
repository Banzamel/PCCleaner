//! Older driver versions in the DriverStore. Third-party packages (`oemNN.inf`) are grouped by their
//! original .inf name, provider and class; each group keeps the newest version and every package a
//! device uses. The rest are removed by `pnputil /delete-driver` without /force, so Windows refuses a
//! package in use anyway (DriverStore Explorer works the same way).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// One removable package.
#[derive(Debug, Clone, PartialEq)]
pub struct DriverPackage {
    /// oemNN.inf
    pub inf: String,
    /// Original file name, e.g. nv_dispi.inf
    pub name: String,
    pub provider: String,
    pub class: String,
    pub version: String,
    pub date: String,
    pub folder: PathBuf,
    pub bytes: f64,
}

/// [Version] fields of an INF file, with %strings% resolved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InfVersion {
    pub provider: String,
    pub class: String,
    pub version: String,
    pub date: String,
}

/// INF files are UTF-16 (with BOM) or ANSI.
pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let u: Vec<u16> = bytes[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&u);
    }
    String::from_utf8_lossy(bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)).to_string()
}

fn sections(text: &str) -> HashMap<String, Vec<(String, String)>> {
    let mut out: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.split(';').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            current = line[1..line.len() - 1].trim().to_lowercase();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            out.entry(current.clone()).or_default().push((k.trim().to_lowercase(), v.trim().trim_matches('"').to_string()));
        }
    }
    out
}

pub fn parse_inf(text: &str) -> InfVersion {
    let s = sections(text);
    let strings: HashMap<String, String> = s
        .iter()
        .filter(|(k, _)| *k == "strings" || k.starts_with("strings."))
        .flat_map(|(_, v)| v.iter().cloned())
        .collect();
    let resolve = |v: &str| {
        let key = v.trim_matches('%').to_lowercase();
        if v.starts_with('%') && v.ends_with('%') { strings.get(&key).cloned().unwrap_or_else(|| v.to_string()) } else { v.to_string() }
    };
    let mut iv = InfVersion::default();
    for (k, v) in s.get("version").into_iter().flatten() {
        match k.as_str() {
            "provider" => iv.provider = resolve(v),
            "class" => iv.class = resolve(v),
            "driverver" => {
                let (date, version) = v.split_once(',').unwrap_or((v.as_str(), ""));
                iv.date = date.trim().to_string();
                iv.version = version.trim().to_string();
            }
            _ => {}
        }
    }
    iv
}

/// "31.0.101.4502" as comparable numbers; an unreadable version sorts as 0.0.
pub fn version_key(v: &str) -> Vec<u64> {
    let parts: Option<Vec<u64>> = v.split('.').map(|p| p.trim().parse().ok()).collect();
    parts.filter(|p| !p.is_empty()).unwrap_or_else(|| vec![0, 0])
}

/// "10/05/2023" (month/day/year) as yyyymmdd for sorting.
fn date_key(d: &str) -> u32 {
    let p: Vec<u32> = d.split('/').filter_map(|x| x.parse().ok()).collect();
    if p.len() == 3 { p[2] * 10_000 + p[0] * 100 + p[1] } else { 0 }
}

/// Picks the packages to remove: all but the newest of each group, never one in use.
pub fn removable(mut all: Vec<DriverPackage>, used: &HashSet<String>) -> Vec<DriverPackage> {
    let mut groups: HashMap<(String, String, String), Vec<DriverPackage>> = HashMap::new();
    for p in all.drain(..) {
        groups.entry((p.name.to_lowercase(), p.provider.clone(), p.class.clone())).or_default().push(p);
    }
    let mut out = Vec::new();
    for (_, mut g) in groups {
        if g.len() < 2 {
            continue;
        }
        g.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)).then(date_key(&b.date).cmp(&date_key(&a.date))));
        out.extend(g.into_iter().skip(1).filter(|p| !used.contains(&p.inf.to_lowercase())));
    }
    out.sort_by(|a, b| b.bytes.total_cmp(&a.bytes));
    out
}

/// Driver store folder of an installed `oemNN.inf` (the .inf file inside FileRepository).
#[cfg(windows)]
pub fn store_location(inf: &Path) -> Option<PathBuf> {
    use windows::core::PCWSTR;
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupGetInfDriverStoreLocationW;
    let name = crate::system::wide(&inf.to_string_lossy());
    let mut buf = [0u16; 1024];
    // SAFETY: valid NUL-terminated name and an output buffer of known size.
    unsafe { SetupGetInfDriverStoreLocationW(PCWSTR(name.as_ptr()), None, PCWSTR::null(), &mut buf, None).ok()? };
    let len = buf.iter().position(|c| *c == 0).unwrap_or(0);
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

#[cfg(not(windows))]
pub fn store_location(_inf: &Path) -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const INF: &str = "[Version]\r\nSignature=\"$Windows NT$\"\r\nProvider=%NV% ; vendor\r\nClass=Display\r\nDriverVer = 10/05/2023,31.0.15.4601\r\n[Strings]\r\nNV=\"NVIDIA\"\r\n";

    #[test]
    fn parses_version_section() {
        let v = parse_inf(INF);
        assert_eq!((v.provider.as_str(), v.class.as_str(), v.version.as_str(), v.date.as_str()), ("NVIDIA", "Display", "31.0.15.4601", "10/05/2023"));
    }

    #[test]
    fn keeps_newest_and_used() {
        let p = |inf: &str, ver: &str| DriverPackage {
            inf: inf.into(), name: "nv_dispi.inf".into(), provider: "NVIDIA".into(), class: "Display".into(),
            version: ver.into(), date: String::new(), folder: PathBuf::new(), bytes: 1.0,
        };
        let used: HashSet<String> = ["oem2.inf".to_string()].into();
        let r = removable(vec![p("oem1.inf", "31.0.15.1"), p("oem2.inf", "30.0.1.1"), p("oem3.inf", "29.0.0.1")], &used);
        assert_eq!(r.iter().map(|x| x.inf.as_str()).collect::<Vec<_>>(), vec!["oem3.inf"]);
    }

    #[test]
    fn decodes_utf16() {
        let bytes: Vec<u8> = [0xFF, 0xFE].into_iter().chain("[Version]".encode_utf16().flat_map(|c| c.to_le_bytes())).collect();
        assert_eq!(decode(&bytes), "[Version]");
    }
}
