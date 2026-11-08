use super::subfolders;
use crate::scan::folder::{AppGroup, FolderCategory};
use crate::text::Text;
use crate::user::UserContext;

/// JetBrains IDEs on their own and opt-in: after deleting indexes the projects re-index for minutes.
pub fn new(u: &UserContext) -> FolderCategory {
    let paths: Vec<_> = subfolders(&u.local_appdata.join("JetBrains"))
        .into_iter()
        .flat_map(|d| [d.join("caches"), d.join("index")])
        .filter(|p| p.exists())
        .collect();
    let processes = vec!["phpstorm64", "webstorm64", "idea64", "pycharm64", "rider64", "clion64", "goland64", "datagrip64", "rubymine64", "rustrover64"];
    let mut c = FolderCategory::new(
        "ide",
        "JetBrains IDE caches and indexes",
        Text::new("Caches and indexes of PhpStorm, WebStorm, IntelliJ etc. (old versions too). The IDE re-indexes projects afterwards. Skipped while the IDE is running."),
        paths.clone(),
    );
    c.opt_in = true;
    c.groups = vec![AppGroup { name: "JetBrains".into(), processes, paths }];
    c
}
