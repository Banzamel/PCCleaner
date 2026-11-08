use std::collections::HashSet;

use super::installed::Installed;
use super::names::{is_skipped_name, plain_name};
use crate::ctx::Ctx;
use crate::scan::leftover::{Confidence, Leftover, LeftoverKind};
use crate::system::registry;
use crate::text::Text;

/// Vendor keys under Software with no installed program and no folder of that name.
pub fn find(ctx: &Ctx, installed: &Installed, folder_names: &HashSet<String>) -> Vec<Leftover> {
    let roots = [format!("{}\\Software", ctx.user.hkcu), "HKEY_LOCAL_MACHINE\\SOFTWARE".into(), "HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node".into()];
    let known = |n: &str| installed.belongs(n, None) || folder_names.contains(&plain_name(n));
    let mut out = Vec::new();
    for root in roots {
        for n in registry::subkeys(&root) {
            if is_skipped_name(&n) || known(&n) {
                continue;
            }
            let key = format!("{root}\\{n}");
            // A studio key (e.g. Klei) may belong to a still-installed game (Klei\Don't Starve)
            let subkeys = registry::subkeys(&key);
            if subkeys.iter().any(|s| known(s)) {
                continue;
            }
            let contains: Vec<String> = subkeys.into_iter().take(3).collect();
            let base = Text::new("Vendor key with no installed program or folder.");
            let reason = if contains.is_empty() {
                base
            } else {
                Text::new("{0} {1}").text(base).text(Text::new("Contains: {0}.").arg(contains.join(", ")))
            };
            let mut l = Leftover::new(LeftoverKind::Registry, key, n, Confidence::Medium, reason);
            l.subkeys = contains;
            out.push(l);
        }
    }
    out
}
