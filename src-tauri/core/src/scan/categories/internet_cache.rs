use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    FolderCategory::new(
        "inet",
        "Windows internet cache (INetCache)",
        Text::new("WebView/IE engine cache. Rebuilds itself."),
        vec![u.local_appdata.join("Microsoft\\Windows\\INetCache")],
    )
}
