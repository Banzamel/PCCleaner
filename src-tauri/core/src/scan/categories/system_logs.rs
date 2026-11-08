use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

/// Only logs older than a week: recent ones may still be needed to diagnose a failed update.
pub fn new(u: &UserContext) -> FolderCategory {
    let mut c = FolderCategory::new(
        "logs",
        "CBS/DISM system logs (older than 7 days)",
        Text::new("Windows component installation logs."),
        vec![u.windir.join("Logs\\CBS"), u.windir.join("Logs\\DISM")],
    );
    c.min_age = 7;
    c
}
