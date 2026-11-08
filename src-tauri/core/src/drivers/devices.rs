use std::collections::HashMap;

use serde::Deserialize;

use super::{problem_description, wmi_date, ProblemDevice};
use crate::system::wmi;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Device {
    pub name: Option<String>,
    #[serde(rename = "DeviceID")]
    pub device_id: Option<String>,
    pub config_manager_error_code: Option<u32>,
    pub present: Option<bool>,
    #[serde(rename = "HardwareID")]
    pub hardware_id: Option<Vec<String>>,
    #[serde(rename = "CompatibleID")]
    pub compatible_id: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignedDriver {
    #[serde(rename = "DeviceID")]
    pub device_id: Option<String>,
    pub driver_version: Option<String>,
    pub driver_date: Option<String>,
}

pub fn all() -> Vec<Device> {
    wmi::query(wmi::CIMV2, "SELECT Name, DeviceID, ConfigManagerErrorCode, Present, HardwareID, CompatibleID FROM Win32_PnPEntity")
}

/// Current driver per device id: (version, date).
pub fn current_drivers() -> HashMap<String, (Option<String>, Option<String>)> {
    wmi::query::<SignedDriver>(wmi::CIMV2, "SELECT DeviceID, DriverVersion, DriverDate FROM Win32_PnPSignedDriver")
        .into_iter()
        .filter_map(|d| Some((d.device_id?, (d.driver_version, d.driver_date.as_deref().and_then(wmi_date)))))
        .collect()
}

/// Devices with an error in Device Manager. Disabled (22) and disconnected (24, 45) are skipped.
pub fn problems(devices: &[Device]) -> Vec<ProblemDevice> {
    devices
        .iter()
        .filter(|d| !matches!(d.config_manager_error_code.unwrap_or(0), 0 | 22 | 24 | 45) && d.present != Some(false))
        .map(|d| {
            let code = d.config_manager_error_code.unwrap_or(0);
            let id = d.device_id.clone().unwrap_or_default();
            ProblemDevice { name: d.name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| id.clone()), id, code, description: problem_description(code) }
        })
        .collect()
}

/// Device by hardware id (HardwareID first, then CompatibleID), lower case.
pub fn by_hardware_id(devices: &[Device]) -> HashMap<String, Device> {
    let mut map = HashMap::new();
    for ids in [|d: &Device| d.hardware_id.clone(), |d: &Device| d.compatible_id.clone()] {
        for d in devices {
            for h in ids(d).unwrap_or_default() {
                map.entry(h.to_lowercase()).or_insert_with(|| d.clone());
            }
        }
    }
    map
}
