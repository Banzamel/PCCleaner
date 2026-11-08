use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{approved, EntryKind, StartupEntry};
use crate::clean::backup;
use crate::error::{AppError, AppResult};
use crate::storage::paths::{safe_name, stamp, DataPaths};
use crate::system::{recycle, registry, tasks};
use crate::text::Text;

fn refuse(e: &StartupEntry, key: &str) -> AppResult<()> {
    match &e.protected {
        Some(why) => Err(AppError::new(Text::new(key).arg(&e.name).text(why.clone()), "protected startup entry")),
        None => Ok(()),
    }
}

/// Turns an entry on or off like Task Manager. Applies at the next sign-in.
pub fn set_enabled(e: &StartupEntry, enable: bool) -> AppResult<()> {
    refuse(e, "{0}: not changed ({1})")?;
    match e.kind {
        EntryKind::Task => tasks::set_enabled(&e.path, enable).map_err(AppError::raw),
        _ => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            registry::set_binary(&e.approved_key, &e.approved_name, &approved::value(enable, now)).map_err(AppError::from)
        }
    }
}

/// Removes an entry permanently, with a backup: a Run value as .reg, a shortcut to the Recycle Bin,
/// a task as .xml. Returns where the backup is.
pub fn remove(e: &StartupEntry, data: &DataPaths) -> AppResult<Text> {
    refuse(e, "{0}: not removed ({1})")?;
    let dir = data.backups(&stamp());
    let file = |ext: &str| dir.join(format!("autostart_{}.{ext}", safe_name(&e.name)));
    let place = match e.kind {
        EntryKind::Run => {
            let f = file("reg");
            backup::export_value(&e.reg_key, &e.name, &f).map_err(|t| AppError::new(t, "backup failed"))?;
            registry::delete_value(&e.reg_key, &e.name)?;
            Text::new("{0}").arg(f.display())
        }
        EntryKind::Folder => {
            recycle::send_to_recycle_bin(Path::new(&e.path)).map_err(AppError::raw)?;
            Text::new("Recycle Bin")
        }
        EntryKind::Task => {
            let f = file("xml");
            backup::export_task(&e.path, &f).map_err(|t| AppError::new(t, "backup failed"))?;
            tasks::delete(&e.path).map_err(AppError::raw)?;
            Text::new("{0}").arg(f.display())
        }
    };
    // The Task Manager on/off state is no longer needed
    if !e.approved_key.is_empty() {
        let _ = registry::delete_value(&e.approved_key, &e.approved_name);
    }
    Ok(place)
}
