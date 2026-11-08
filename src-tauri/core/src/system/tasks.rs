//! Task Scheduler through its COM API. Tasks under `\Microsoft\` belong to Windows and are never listed.
//! Callers hold a [`super::ComGuard`].

/// A Task Scheduler task outside `\Microsoft\`.
#[derive(Debug, Clone, Default)]
pub struct TaskInfo {
    pub name: String,
    /// Folder path such as `\` or `\Vendor\`.
    pub folder: String,
    /// Full path `\Vendor\Name`.
    pub path: String,
    pub enabled: bool,
    /// Has a sign-in or boot trigger (a startup app).
    pub at_startup: bool,
    /// (program, arguments) of each "start a program" action.
    pub actions: Vec<(String, String)>,
}

/// Splits `\Vendor\Name` into (`\Vendor`, `Name`); the root folder is `\`.
pub fn split_path(path: &str) -> (String, String) {
    match path.rfind('\\') {
        Some(0) | None => ("\\".into(), path.trim_start_matches('\\').into()),
        Some(i) => (path[..i].into(), path[i + 1..].into()),
    }
}

#[cfg(windows)]
mod imp {
    use super::TaskInfo;
    use windows::core::{Interface, BSTR};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::Foundation::VARIANT_BOOL;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::TaskScheduler::*;

    fn service() -> windows::core::Result<ITaskService> {
        // SAFETY: standard COM activation and connection to the local service.
        unsafe {
            let s: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
            let none = VARIANT::default();
            s.Connect(&none, &none, &none, &none)?;
            Ok(s)
        }
    }

    fn folder(s: &ITaskService, path: &str) -> windows::core::Result<ITaskFolder> {
        // SAFETY: valid BSTR argument.
        unsafe { s.GetFolder(&BSTR::from(path)) }
    }

    fn read(t: &IRegisteredTask) -> windows::core::Result<TaskInfo> {
        // SAFETY: COM calls on a live task object; out-parameters are locals.
        unsafe {
            let path = t.Path()?.to_string();
            let (folder, name) = super::split_path(&path);
            let def = t.Definition()?;
            let triggers = def.Triggers()?;
            let mut count = 0;
            triggers.Count(&mut count)?;
            let mut at_startup = false;
            for i in 1..=count {
                let mut kind = TASK_TRIGGER_TYPE2::default();
                triggers.get_Item(i)?.Type(&mut kind)?;
                at_startup |= kind == TASK_TRIGGER_LOGON || kind == TASK_TRIGGER_BOOT;
            }
            let actions = def.Actions()?;
            actions.Count(&mut count)?;
            let mut list = Vec::new();
            for i in 1..=count {
                let Ok(exec) = actions.get_Item(i)?.cast::<IExecAction>() else { continue };
                let (mut p, mut a) = (BSTR::new(), BSTR::new());
                exec.Path(&mut p)?;
                exec.Arguments(&mut a)?;
                list.push((p.to_string(), a.to_string()));
            }
            let enabled = t.State()? != TASK_STATE_DISABLED;
            Ok(TaskInfo { name, folder: if folder.ends_with('\\') { folder } else { format!("{folder}\\") }, path, enabled, at_startup, actions: list })
        }
    }

    fn walk(f: &ITaskFolder, out: &mut Vec<TaskInfo>) -> windows::core::Result<()> {
        // SAFETY: COM collection access with 1-based indexes as the API requires.
        unsafe {
            if f.Path()?.to_string().to_lowercase().starts_with("\\microsoft") {
                return Ok(());
            }
            let tasks = f.GetTasks(TASK_ENUM_HIDDEN.0)?;
            for i in 1..=tasks.Count()? {
                if let Ok(info) = tasks.get_Item(&VARIANT::from(i)).and_then(|t| read(&t)) {
                    out.push(info);
                }
            }
            let subs = f.GetFolders(0)?;
            for i in 1..=subs.Count()? {
                if let Ok(sub) = subs.get_Item(&VARIANT::from(i)) {
                    let _ = walk(&sub, out);
                }
            }
        }
        Ok(())
    }

    pub fn list_third_party() -> Vec<TaskInfo> {
        let mut out = Vec::new();
        if let Ok(s) = service() {
            if let Ok(root) = folder(&s, "\\") {
                let _ = walk(&root, &mut out);
            }
        }
        out
    }

    fn task(path: &str) -> windows::core::Result<(ITaskFolder, IRegisteredTask, String)> {
        let (dir, name) = super::split_path(path);
        let f = folder(&service()?, &dir)?;
        // SAFETY: valid BSTR name in that folder.
        let t = unsafe { f.GetTask(&BSTR::from(name.as_str()))? };
        Ok((f, t, name))
    }

    pub fn set_enabled(path: &str, enable: bool) -> Result<(), String> {
        let (_, t, _) = task(path).map_err(|e| e.message())?;
        // SAFETY: live task object.
        unsafe { t.SetEnabled(VARIANT_BOOL::from(enable)) }.map_err(|e| e.message())
    }

    pub fn export_xml(path: &str) -> Result<String, String> {
        let (_, t, _) = task(path).map_err(|e| e.message())?;
        // SAFETY: live task object.
        unsafe { t.Xml() }.map(|x| x.to_string()).map_err(|e| e.message())
    }

    pub fn delete(path: &str) -> Result<(), String> {
        let (f, _, name) = task(path).map_err(|e| e.message())?;
        // SAFETY: deleting a task by name in its own folder.
        unsafe { f.DeleteTask(&BSTR::from(name.as_str()), 0) }.map_err(|e| e.message())
    }

    pub fn exists(path: &str) -> bool {
        task(path).is_ok()
    }

    /// Creates or replaces a task in the root folder from its XML, for the signed-in account.
    pub fn register_xml(name: &str, xml: &str) -> Result<(), String> {
        let s = service().map_err(|e| e.message())?;
        let root = folder(&s, "\\").map_err(|e| e.message())?;
        let none = VARIANT::default();
        // SAFETY: valid BSTRs; TASK_CREATE_OR_UPDATE with the interactive token of the current user.
        unsafe {
            root.RegisterTask(&BSTR::from(name), &BSTR::from(xml), TASK_CREATE_OR_UPDATE.0, &none, &none, TASK_LOGON_INTERACTIVE_TOKEN, &none)
        }
        .map(|_| ())
        .map_err(|e| e.message())
    }
}

#[cfg(windows)]
pub use imp::{delete, export_xml, exists, list_third_party, register_xml, set_enabled};

#[cfg(not(windows))]
mod fallback {
    use super::TaskInfo;
    pub fn list_third_party() -> Vec<TaskInfo> { Vec::new() }
    pub fn set_enabled(_: &str, _: bool) -> Result<(), String> { Err("unsupported".into()) }
    pub fn export_xml(_: &str) -> Result<String, String> { Err("unsupported".into()) }
    pub fn delete(_: &str) -> Result<(), String> { Err("unsupported".into()) }
    pub fn exists(_: &str) -> bool { false }
    pub fn register_xml(_: &str, _: &str) -> Result<(), String> { Err("unsupported".into()) }
}
#[cfg(not(windows))]
pub use fallback::*;

#[cfg(test)]
mod tests {
    #[test]
    fn split() {
        assert_eq!(super::split_path("\\Vendor\\Update"), ("\\Vendor".into(), "Update".into()));
        assert_eq!(super::split_path("\\Root task"), ("\\".into(), "Root task".into()));
    }
}
