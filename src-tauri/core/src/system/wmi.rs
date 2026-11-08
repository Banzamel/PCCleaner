use serde::de::DeserializeOwned;

/// Runs a WQL query (e.g. devices, drivers, shadow copies) and returns no rows on any error, like
/// the old program's `Get-CimInstance` with errors silenced.
#[cfg(windows)]
pub fn query<T: DeserializeOwned>(namespace: &str, wql: &str) -> Vec<T> {
    let Ok(con) = wmi::WMIConnection::with_namespace_path(namespace) else { return Vec::new() };
    con.raw_query(wql).unwrap_or_default()
}

#[cfg(not(windows))]
pub fn query<T: DeserializeOwned>(_namespace: &str, _wql: &str) -> Vec<T> {
    Vec::new()
}

pub const CIMV2: &str = "ROOT\\CIMV2";
pub const DEFAULT: &str = "ROOT\\DEFAULT";
