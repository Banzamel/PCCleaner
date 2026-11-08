use std::path::{Path, PathBuf};

use super::subfolders;
use crate::scan::folder::{AppGroup, FolderCategory};
use crate::system::registry;
use crate::text::Text;
use crate::user::UserContext;

/// Electron apps keep their caches in the same sub-folders.
fn electron(dir: &Path) -> Vec<PathBuf> {
    ["Cache", "Code Cache", "GPUCache", "DawnCache", "Service Worker\\CacheStorage"].iter().map(|c| dir.join(c)).collect()
}

fn group(name: &str, processes: &[&'static str], paths: Vec<PathBuf>) -> AppGroup {
    AppGroup { name: name.to_string(), processes: processes.to_vec(), paths }
}

/// A list of names as one text: each name is translated on its own ("Teams (classic)").
pub fn name_list(names: &[String]) -> Text {
    let key = (0..names.len()).map(|i| format!("{{{i}}}")).collect::<Vec<_>>().join(", ");
    names.iter().fold(Text::new(&key), |t, n| t.text(Text::new(n)))
}

pub fn new(u: &UserContext) -> FolderCategory {
    let roaming = &u.appdata;
    let local = &u.local_appdata;
    let steam = registry::read_string(&format!("{}\\Software\\Valve\\Steam", u.hkcu), "SteamPath").map(|s| PathBuf::from(s.replace('/', "\\")));
    let mut steam_paths: Vec<PathBuf> = steam.iter().flat_map(|s| [s.join("appcache\\httpcache"), s.join("appcache\\librarycache")]).collect();
    steam_paths.push(local.join("Steam\\htmlcache"));
    let epic: Vec<PathBuf> = subfolders(&local.join("EpicGamesLauncher\\Saved"))
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().to_lowercase().starts_with("webcache")))
        .collect();
    let mut code = electron(&roaming.join("Code"));
    code.extend([roaming.join("Code\\CachedData"), roaming.join("Code\\CachedExtensionVSIXs")]);
    let groups: Vec<AppGroup> = vec![
        group("Discord", &["discord"], electron(&roaming.join("discord"))),
        group("Slack", &["slack"], electron(&roaming.join("Slack"))),
        group("Teams (classic)", &["teams"], electron(&roaming.join("Microsoft\\Teams"))),
        group("VS Code", &["code"], code),
        group("Spotify", &["spotify"], vec![local.join("Spotify\\Data")]),
        group("Steam", &["steam", "steamwebhelper"], steam_paths),
        group("Epic Games Launcher", &["epicgameslauncher"], epic),
        group("Battle.net", &["battle.net"], vec![local.join("Battle.net\\BrowserCaches"), local.join("Battle.net\\Cache")]),
    ]
    .into_iter()
    // Only apps present on this computer
    .map(|mut g| {
        g.paths.retain(|p| p.exists());
        g
    })
    .filter(|g| !g.paths.is_empty())
    .collect();
    let names: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();
    let found = if names.is_empty() { Text::new("none") } else { name_list(&names) };
    let mut c = FolderCategory::new(
        "apps",
        "App caches",
        Text::new("Cache only; accounts and settings are kept. Running apps are skipped. Found: {0}.").text(found),
        groups.iter().flat_map(|g| g.paths.clone()).collect(),
    );
    c.groups = groups;
    c
}

#[cfg(test)]
mod tests {
    #[test]
    fn name_list_key() {
        let t = super::name_list(&["Steam".into(), "Teams (classic)".into()]);
        assert_eq!(t.key, "{0}, {1}");
        assert_eq!(t.english(), "Steam, Teams (classic)");
    }
}
