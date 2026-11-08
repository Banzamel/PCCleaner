//! Name rules. Detection is cautious: a folder is a leftover only if its name matches NO installed
//! program, Store app, service, process or Start menu shortcut.

use std::sync::LazyLock;

use regex::Regex;

/// Words too common to tell programs apart.
pub const COMMON_WORDS: &[&str] = &[
    "app", "apps", "data", "studio", "software", "soft", "games", "game", "the", "inc", "ltd", "llc", "gmbh", "corp", "corporation",
    "company", "technologies", "technology", "systems", "group", "limited", "edition", "launcher", "client", "free", "pro", "update",
    "updater", "helper", "service", "services", "tools", "tool", "common", "files", "program", "programs", "user", "local", "roaming",
    "cache", "x64", "x86", "win", "windows", "desktop", "setup", "install", "installer", "version", "sro", "foundation",
    "entertainment", "interactive", "digital",
];

/// System folders and keys, drivers and tools that are never suggested (wildcards as in PowerShell -like).
pub const SKIPPED_NAMES: &[&str] = &[
    "microsoft*", "windows*", "packages", "temp", "tmp", "programs", "package cache", "common files", "installshield*", "d3dscache",
    "crashdumps", "connecteddevicesplatform", "peerdistrepub", "placeholdertilelogofolder", "comms", "publishers", "virtualstore",
    "isolatedstorage", "elevateddiagnostics", "squirreltemp", "history", "temporary internet files", "application data", "fontcache",
    "identities", "msbuild", "reference assemblies", "uninstall information", "internet explorer", "modifiablewindowsapps", "dotnet",
    "ssh", "regid.*", "uso*", "softwaredistribution", "desktop", "documents", "start menu", "templates", "favorites", "npm",
    "npm-cache", "pip", "uv", "yarn", "nuget", "node-gyp", "pnpm*", "fontconfig", "nvidia*", "intel*", "amd", "ati*", "realtek",
    "dell", "hp", "lenovo", "asus*", "msi", "gigabyte", "claude*", "anthropic*", "chocolatey", "scoop", "classes", "policies",
    "wow6432node", "clients", "registeredapplications", "odbc", "khronos", "partner", "defaultuserenvironment", "setup", "oem",
    "appdatalow", "description", "hardware", "system", "mozillaplugins", "chromium", "dolby", "waves*", "conexant", "synaptics",
    "elantech", "pccleaner", "pc cleaner", "ollama", "ms-playwright", "gtk*", "cache", "logs", "crashpad", "javasoft",
    "steam controller configs", "steamworks shared", ".*", "pl.banzamel.pccleaner",
];

static CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new("([a-z])([A-Z])").expect("valid regex"));
static COMMAND_FILE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^(.+?\.(exe|dll|ico|bat|cmd))").expect("valid regex"));

/// Case-insensitive match with `*` and `?`, like PowerShell's -like.
pub fn like(text: &str, pattern: &str) -> bool {
    fn go(t: &[char], p: &[char]) -> bool {
        match p.split_first() {
            None => t.is_empty(),
            Some(('*', rest)) => (0..=t.len()).any(|i| go(&t[i..], rest)),
            Some(('?', rest)) => !t.is_empty() && go(&t[1..], rest),
            Some((c, rest)) => t.first() == Some(c) && go(&t[1..], rest),
        }
    }
    let t: Vec<char> = text.to_lowercase().chars().collect();
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    go(&t, &p)
}

/// Words of a name: "SomeVendor Tools-2" -> ["some", "vendor"].
pub fn name_tokens(t: &str) -> Vec<String> {
    let spaced = CAMEL.replace_all(t, "$1 $2").to_lowercase();
    spaced
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 3 && !COMMON_WORDS.contains(w) && !w.chars().all(|c| c.is_ascii_digit()))
        .map(String::from)
        .collect()
}

/// Letters and digits only, lower case.
pub fn plain_name(t: &str) -> String {
    t.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

pub fn is_skipped_name(name: &str) -> bool {
    SKIPPED_NAMES.iter().any(|p| like(name, p))
}

/// File path from a command such as `"C:\X\unins.exe" /S` or `C:\X\app.exe,0`.
pub fn command_path(s: &str) -> Option<String> {
    let s = crate::system::registry::expand_env(s.trim());
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            if end > 0 {
                return Some(rest[..end].to_string());
            }
        }
    }
    if let Some(m) = COMMAND_FILE.captures(&s) {
        return Some(m[1].to_string());
    }
    Some(s.split(',').next().unwrap_or("").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards() {
        assert!(like("Microsoft Edge", "microsoft*"));
        assert!(like(".cache", ".*"));
        assert!(like("regid.1991-06.com.microsoft", "regid.*"));
        assert!(!like("Steam", "temp"));
    }

    #[test]
    fn tokens_and_plain_names() {
        assert_eq!(name_tokens("KleiEntertainment Game-Tools 2024"), vec!["klei"]);
        assert_eq!(plain_name("Don't Starve!"), "dontstarve");
        assert!(is_skipped_name("NVIDIA Corporation"));
        assert!(!is_skipped_name("Klei"));
    }

    #[test]
    fn command_paths() {
        assert_eq!(command_path("\"C:\\X Y\\unins000.exe\" /SILENT").as_deref(), Some("C:\\X Y\\unins000.exe"));
        assert_eq!(command_path("C:\\X\\app.exe,0").as_deref(), Some("C:\\X\\app.exe"));
        assert_eq!(command_path("C:\\X\\icon.png,1").as_deref(), Some("C:\\X\\icon.png"));
        assert_eq!(command_path("  "), None);
    }
}
