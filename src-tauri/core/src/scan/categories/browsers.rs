use super::subfolders;
use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

/// Page and shader caches of every profile. Passwords, bookmarks, history and sessions live
/// elsewhere in the profile and are never touched.
pub fn new(u: &UserContext) -> FolderCategory {
    let mut paths = Vec::new();
    for ud in ["Google\\Chrome\\User Data", "Microsoft\\Edge\\User Data", "BraveSoftware\\Brave-Browser\\User Data"] {
        let ud = u.local_appdata.join(ud);
        for p in subfolders(&ud) {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if name == "Default" || name.starts_with("Profile ") {
                paths.extend(["Cache", "Code Cache", "GPUCache", "Service Worker\\CacheStorage"].map(|c| p.join(c)));
            }
        }
        paths.extend([ud.join("GrShaderCache"), ud.join("ShaderCache")]);
    }
    for p in subfolders(&u.local_appdata.join("Mozilla\\Firefox\\Profiles")) {
        paths.push(p.join("cache2"));
    }
    let mut c = FolderCategory::new(
        "www",
        "Browser caches (Chrome, Edge, Brave, Firefox)",
        Text::new("Page and shader caches only. Passwords, bookmarks, history and sessions are kept."),
        paths,
    );
    c.processes = vec!["chrome", "msedge", "brave", "firefox"];
    c
}
