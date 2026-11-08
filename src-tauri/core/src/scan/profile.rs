use std::collections::HashMap;

use serde::Serialize;

use super::leftovers::uninstall;
use crate::ctx::Ctx;
use crate::fs::link::CLOUD_ONLY;
use crate::fs::walk;
use crate::system::drives;
use crate::text::Text;

const GB: u64 = 1 << 30;
const MB: u64 = 1 << 20;

/// A large file the user may want to delete by hand (never selected in advance).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub path: String,
    pub bytes: f64,
    pub modified: Option<std::time::SystemTime>,
    pub in_downloads: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedSize {
    pub name: String,
    pub bytes: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileScan {
    #[serde(skip)]
    pub candidates: Vec<Candidate>,
    pub profile: Vec<NamedSize>,
    pub appdata: Vec<NamedSize>,
    pub programs: Vec<NamedSize>,
    /// hiberfil.sys size (None = hibernation off)
    pub hiberfil: Option<f64>,
    pub windows_old: bool,
    pub old_setup_bt: bool,
    pub drives: Vec<drives::DriveInfo>,
}

fn top(map: HashMap<String, u64>, min: u64, n: usize) -> Vec<NamedSize> {
    let mut v: Vec<_> = map.into_iter().filter(|(_, b)| *b >= min).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    v.into_iter().take(n).map(|(name, b)| NamedSize { name, bytes: b as f64, installed: None }).collect()
}

/// Large files and the biggest folders in the profile, the biggest programs and system files - facts
/// for the plan and the AI report.
pub fn scan(ctx: &Ctx) -> ProfileScan {
    let profile = &ctx.user.profile;
    let prefix = profile.to_string_lossy().len() + 1;
    let (mut top_folders, mut appdata): (HashMap<String, u64>, HashMap<String, u64>) = Default::default();
    let mut large = Vec::new();
    let mut n = 0u64;
    for f in walk::files(profile, ctx.excluded) {
        n += 1;
        if n % 5000 == 0 {
            ctx.out.status(Text::new("scanned {0:N0} files...").arg(n));
        }
        if f.attrs & CLOUD_ONLY != 0 {
            continue; // cloud-only files (OneDrive) - reading them would download them
        }
        let full = f.path.to_string_lossy().to_string();
        let parts: Vec<&str> = full.get(prefix..).unwrap_or("").split(['\\', '/']).collect();
        if parts.len() < 2 {
            continue;
        }
        *top_folders.entry(parts[0].to_string()).or_default() += f.len;
        if parts[0] == "AppData" {
            if parts.len() > 3 && (parts[1] == "Local" || parts[1] == "Roaming") {
                *appdata.entry(format!("AppData\\{}\\{}", parts[1], parts[2])).or_default() += f.len;
            }
        } else if f.len >= GB {
            large.push(f);
        }
    }
    ctx.out.status(Text::new("scanned {0:N0} files in the profile.").arg(n));

    let downloads = ctx.user.downloads();
    let dl_prefix = format!("{}\\", downloads.to_string_lossy().to_lowercase());
    let mut from_downloads: Vec<_> = walk::files(&downloads, ctx.excluded).filter(|f| f.len >= 100 * MB).collect();
    from_downloads.sort_by(|a, b| b.len.cmp(&a.len));
    large.retain(|f| !f.path.to_string_lossy().to_lowercase().starts_with(&dl_prefix));
    large.sort_by(|a, b| b.len.cmp(&a.len));
    let mut candidates: Vec<Candidate> = Vec::new();
    for (f, in_dl) in from_downloads.into_iter().take(15).map(|f| (f, true)).chain(large.into_iter().take(12).map(|f| (f, false))) {
        candidates.push(Candidate { path: f.path.to_string_lossy().to_string(), bytes: f.len as f64, modified: f.modified, in_downloads: in_dl });
    }

    let mut programs: Vec<_> = uninstall::read_all(&ctx.user.hkcu).into_iter().filter(|e| e.estimated_size.is_some_and(|s| s > 0.0)).collect();
    programs.sort_by(|a, b| b.estimated_size.unwrap_or(0.0).total_cmp(&a.estimated_size.unwrap_or(0.0)));
    let programs = programs
        .into_iter()
        .take(12)
        .map(|e| NamedSize { name: e.display_name, bytes: e.estimated_size.unwrap_or(0.0) * 1024.0, installed: Some(e.install_date).filter(|d| !d.is_empty()) })
        .collect();

    let all_drives = drives::fixed_drives(&ctx.user.system_drive);
    ProfileScan {
        candidates,
        profile: top(top_folders, 0, 10),
        appdata: top(appdata, 300 * MB, 12),
        programs,
        hiberfil: std::fs::metadata(ctx.user.sys_path("hiberfil.sys")).ok().map(|m| m.len() as f64),
        windows_old: ctx.user.sys_path("Windows.old").exists(),
        old_setup_bt: ctx.user.sys_path("$WINDOWS.~BT").exists(),
        drives: all_drives.into_iter().filter(|d| ctx.drives.iter().any(|s| s.eq_ignore_ascii_case(&d.letter))).collect(),
    }
}
