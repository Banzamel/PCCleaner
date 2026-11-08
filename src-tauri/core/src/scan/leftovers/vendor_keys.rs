use std::collections::HashSet;

use super::installed::Installed;
use super::names::{is_skipped_name, plain_name};
use super::vendor_rules::{is_guid_name, matches_uninstall_entry, points_to_existing, uninstall_names};
use crate::ctx::Ctx;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::system::registry;
use crate::text::Text;

/// Names of every Uninstall subkey, also those without a DisplayName.
fn all_uninstall_names(hkcu: &str) -> HashSet<String> {
    let roots = [
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
        "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string(),
        format!("{hkcu}\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];
    uninstall_names(roots.iter().flat_map(|r| registry::subkeys(r)))
}

/// String values of a key (InstallLocation, Path...).
fn string_values(key: &str) -> Vec<String> {
    registry::value_names(key).iter().filter_map(|n| registry::read_string(key, n)).collect()
}

/// Vendor keys under Software with no installed program and no folder of that name.
pub fn find(ctx: &Ctx, installed: &Installed, folder_names: &HashSet<String>) -> Vec<Leftover> {
    let roots = [format!("{}\\Software", ctx.user.hkcu), "HKEY_LOCAL_MACHINE\\SOFTWARE".into(), "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node".into()];
    let uninstall = all_uninstall_names(&ctx.user.hkcu);
    let known = |n: &str| installed.belongs(n, None) || folder_names.contains(&plain_name(n)) || matches_uninstall_entry(n, &uninstall);
    let exists = |p: &str| std::path::Path::new(p).exists();
    let mut out = Vec::new();
    for root in roots {
        for n in registry::subkeys(&root) {
            if is_skipped_name(&n) || known(&n) {
                continue;
            }
            let key = format!("{root}\\{n}");
            // A per-app key with a path to a program that is still there (electron-builder keeps InstallLocation)
            if points_to_existing(&string_values(&key), exists) {
                continue;
            }
            // A studio key (e.g. Klei) may belong to a still-installed game (Klei\Don't Starve)
            let subkeys = registry::subkeys(&key);
            if subkeys.iter().any(|s| known(s) || points_to_existing(&string_values(&format!("{key}\\{s}")), exists)) {
                continue;
            }
            let contains: Vec<String> = subkeys.into_iter().take(3).collect();
            let base = Text::new("Vendor key with no installed program or folder.");
            let reason = if contains.is_empty() {
                base
            } else {
                Text::new("{0} {1}").text(base).text(Text::new("Contains: {0}.").arg(contains.join(", ")))
            };
            // A GUID name tells nothing about the program: listed, but never selected in advance
            let confidence = if is_guid_name(&n) { Confidence::Low } else { Confidence::Medium };
            let mut l = Leftover::new(LeftoverKind::Registry, key, n, confidence, reason);
            l.subkeys = contains;
            out.push(l);
        }
    }
    out
}
