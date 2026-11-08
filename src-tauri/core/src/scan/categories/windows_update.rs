use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

/// The Windows Update and BITS services hold these files; they are stopped while the folder is emptied.
pub fn new(u: &UserContext) -> FolderCategory {
    let mut c = FolderCategory::new(
        "wu",
        "Windows Update cache",
        Text::new("Already downloaded update files. Windows downloads them again if needed."),
        vec![u.windir.join("SoftwareDistribution\\Download"), u.windir.join("SoftwareDistribution\\DeliveryOptimization")],
    );
    c.services = vec!["wuauserv", "bits"];
    c
}
