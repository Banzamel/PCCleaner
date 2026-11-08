use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    let l = &u.local_appdata;
    FolderCategory::new(
        "shaders",
        "GPU shader caches (NVIDIA, DirectX)",
        Text::new("Compiled shaders. Games may load longer the first time after cleaning."),
        vec![l.join("NVIDIA\\DXCache"), l.join("NVIDIA\\GLCache"), l.join("D3DSCache")],
    )
}
