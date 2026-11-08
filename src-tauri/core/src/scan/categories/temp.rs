use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    FolderCategory::new(
        "temp",
        "Temporary files (TEMP)",
        Text::new("Temporary files of programs and installers. Files in use are skipped."),
        vec![u.temp.clone(), u.windir.join("Temp")],
    )
}
