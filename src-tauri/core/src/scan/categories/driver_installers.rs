use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    FolderCategory::new(
        "drivers",
        "Old driver installers (NVIDIA, AMD)",
        Text::new("Extracted and downloaded driver installers. The installed driver works without them."),
        vec![u.sys_path("NVIDIA"), u.sys_path("AMD"), u.program_data.join("NVIDIA Corporation\\Downloader")],
    )
}
