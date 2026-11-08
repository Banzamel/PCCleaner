//! Windows Update Agent (COM): searching and installing driver updates one by one.
//! Callers hold a [`crate::system::ComGuard`]. Search usually takes 1-3 minutes.

use super::DriverCheck;

/// Install state of one driver, reported while installing.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallState {
    Waiting,
    Downloading,
    Installing,
    Ok,
    Restart,
    Gone,
    Fail(String),
}

impl InstallState {
    pub fn code(&self) -> &'static str {
        match self {
            InstallState::Waiting => "waiting",
            InstallState::Downloading => "downloading",
            InstallState::Installing => "installing",
            InstallState::Ok => "ok",
            InstallState::Restart => "restart",
            InstallState::Gone => "gone",
            InstallState::Fail(_) => "fail",
        }
    }
}

// The Windows Update constants keep their API names (orcSucceeded...)
#[cfg(windows)]
#[allow(non_upper_case_globals)]
mod imp {
    use super::super::{devices, ole_date, problem_description, version_from_title, DriverUpdate};
    use super::*;
    use windows::core::{Interface, BSTR};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::UpdateAgent::*;

    fn session() -> windows::core::Result<IUpdateSession> {
        // SAFETY: standard COM activation.
        unsafe {
            let s: IUpdateSession = CoCreateInstance(&UpdateSession, None, CLSCTX_INPROC_SERVER)?;
            s.SetClientApplicationID(&BSTR::from("PC Cleaner"))?;
            Ok(s)
        }
    }

    /// Updates matching the criteria, from Windows Update even on a computer with WSUS.
    fn search(s: &IUpdateSession, criteria: &str) -> windows::core::Result<Vec<IUpdate>> {
        // SAFETY: COM calls on live objects; 0-based collection indexes.
        unsafe {
            let searcher = s.CreateUpdateSearcher()?;
            searcher.SetServerSelection(ssWindowsUpdate)?;
            let found = searcher.Search(&BSTR::from(criteria))?.Updates()?;
            (0..found.Count()?).map(|i| found.get_Item(i)).collect()
        }
    }

    fn decimal(d: windows::Win32::Foundation::DECIMAL) -> f64 {
        // SAFETY: reading the plain-data union fields of DECIMAL.
        let (hi, lo, scale) = unsafe { (d.Hi32, d.Anonymous2.Lo64, d.Anonymous1.Anonymous.scale) };
        ((u128::from(hi) << 64) | u128::from(lo)) as f64 / 10f64.powi(i32::from(scale))
    }

    pub fn find() -> Result<DriverCheck, String> {
        let s = session().map_err(|e| e.message())?;
        let updates = search(&s, "IsInstalled=0 and Type='Driver' and IsHidden=0").map_err(|e| e.message())?;
        let all = devices::all();
        let by_hw = devices::by_hardware_id(&all);
        let current = devices::current_drivers();
        let mut drivers = Vec::new();
        for u in updates {
            // SAFETY: COM property reads on a live update object.
            let d = unsafe {
                let Ok(drv) = u.cast::<IWindowsDriverUpdate>() else { continue };
                let title = u.Title().map(|t| t.to_string()).unwrap_or_default();
                let model = drv.DriverModel().map(|t| t.to_string()).unwrap_or_default();
                let hw = drv.DriverHardwareID().map(|t| t.to_string().to_lowercase()).unwrap_or_default();
                let dev = by_hw.get(&hw);
                let (cur_v, cur_d) = dev.and_then(|d| d.device_id.as_ref()).and_then(|id| current.get(id)).cloned().unwrap_or((None, None));
                let code = dev.and_then(|d| d.config_manager_error_code).unwrap_or(0);
                let name = dev.and_then(|d| d.name.clone()).filter(|n| !n.is_empty()).or_else(|| Some(model.clone()).filter(|m| !m.is_empty())).unwrap_or_else(|| title.clone());
                DriverUpdate {
                    id: u.Identity().and_then(|i| i.UpdateID()).map(|t| t.to_string()).unwrap_or_default(),
                    version: version_from_title(&title),
                    name,
                    model,
                    provider: drv.DriverProvider().map(|t| t.to_string()).unwrap_or_default(),
                    class: drv.DriverClass().map(|t| t.to_string()).unwrap_or_default(),
                    bytes: u.MaxDownloadSize().map(decimal).unwrap_or(0.0),
                    new_date: drv.DriverVerDate().ok().and_then(ole_date),
                    current_version: cur_v,
                    current_date: cur_d,
                    problem: (dev.is_some() && !matches!(code, 0 | 22)).then(|| problem_description(code)),
                    title,
                }
            };
            drivers.push(d);
        }
        Ok(DriverCheck { drivers, problems: devices::problems(&all) })
    }

    fn hr(code: i32) -> String {
        format!("0x{:08X}", code as u32)
    }

    fn install_one(s: &IUpdateSession, u: &IUpdate, report: &dyn Fn(InstallState)) -> windows::core::Result<InstallState> {
        // SAFETY: COM calls on live objects created in this function.
        unsafe {
            if !u.EulaAccepted()?.as_bool() {
                u.AcceptEula()?;
            }
            let col: IUpdateCollection = CoCreateInstance(&UpdateCollection, None, CLSCTX_INPROC_SERVER)?;
            col.Add(u)?;
            report(InstallState::Downloading);
            let dl = s.CreateUpdateDownloader()?;
            dl.SetUpdates(&col)?;
            let r = dl.Download()?;
            if !matches!(r.ResultCode()?, orcSucceeded | orcSucceededWithErrors) {
                return Ok(InstallState::Fail(hr(r.GetUpdateResult(0)?.HResult()?)));
            }
            report(InstallState::Installing);
            let inst = s.CreateUpdateInstaller()?;
            inst.SetUpdates(&col)?;
            let r = inst.Install()?;
            if !matches!(r.ResultCode()?, orcSucceeded | orcSucceededWithErrors) {
                return Ok(InstallState::Fail(hr(r.GetUpdateResult(0)?.HResult()?)));
            }
            Ok(if r.RebootRequired()?.as_bool() { InstallState::Restart } else { InstallState::Ok })
        }
    }

    pub fn install(ids: &[String], report: &dyn Fn(&str, InstallState)) {
        let found = session().and_then(|s| search(&s, "IsInstalled=0 and Type='Driver'").map(|u| (s, u)));
        let (s, updates) = match found {
            Ok(v) => v,
            Err(e) => return ids.iter().for_each(|id| report(id, InstallState::Fail(e.message()))),
        };
        // SAFETY: reading the identity of live update objects.
        let id_of = |u: &IUpdate| unsafe { u.Identity().and_then(|i| i.UpdateID()).map(|t| t.to_string()).unwrap_or_default() };
        let pick: Vec<(String, Option<&IUpdate>)> = ids.iter().map(|id| (id.clone(), updates.iter().find(|u| id_of(u) == *id))).collect();
        for (id, u) in &pick {
            report(id, if u.is_some() { InstallState::Waiting } else { InstallState::Gone });
        }
        for (id, u) in pick {
            let Some(u) = u else { continue };
            let state = install_one(&s, u, &|st| report(&id, st)).unwrap_or_else(|e| InstallState::Fail(e.message()));
            report(&id, state);
        }
    }
}

#[cfg(windows)]
pub use imp::{find, install};

#[cfg(not(windows))]
pub fn find() -> Result<DriverCheck, String> {
    Err("Windows Update is available only on Windows".into())
}

#[cfg(not(windows))]
pub fn install(ids: &[String], report: &dyn Fn(&str, InstallState)) {
    ids.iter().for_each(|id| report(id, InstallState::Fail("unsupported".into())));
}
