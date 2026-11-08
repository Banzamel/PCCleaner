use serde::Deserialize;

use crate::system::wmi;

/// WMI returns 64-bit numbers as strings.
#[derive(Deserialize)]
#[serde(untagged)]
enum Num {
    N(u64),
    S(String),
}

impl Num {
    fn value(&self) -> f64 {
        match self {
            Num::N(n) => *n as f64,
            Num::S(s) => s.parse().unwrap_or(0.0),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Volume {
    device_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShadowStorage {
    volume: String,
    used_space: Num,
}

/// The volume GUID part of a device id or of a WMI reference to it.
pub fn volume_guid(s: &str) -> Option<String> {
    let start = s.find("Volume{")?;
    let end = s[start..].find('}')? + start + 1;
    Some(s[start..end].to_lowercase())
}

/// Space used by restore points and shadow copies on a drive (`C:`).
pub fn used_on(drive: &str) -> f64 {
    let vols: Vec<Volume> = wmi::query(wmi::CIMV2, &format!("SELECT DeviceID FROM Win32_Volume WHERE DriveLetter = '{drive}'"));
    let Some(guid) = vols.first().and_then(|v| volume_guid(&v.device_id)) else { return 0.0 };
    let storage: Vec<ShadowStorage> = wmi::query(wmi::CIMV2, "SELECT Volume, UsedSpace FROM Win32_ShadowStorage");
    storage.iter().filter(|s| volume_guid(&s.volume).as_deref() == Some(guid.as_str())).map(|s| s.used_space.value()).sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn guid_from_reference() {
        let r = "Win32_Volume.DeviceID=\"\\\\\\\\?\\\\Volume{AB-12}\\\\\"";
        assert_eq!(super::volume_guid(r).as_deref(), Some("volume{ab-12}"));
    }
}
