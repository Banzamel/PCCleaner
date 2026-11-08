use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    let local = ["npm-cache", "pip\\cache", "node-gyp\\Cache", "Yarn\\Cache", "uv\\cache", "NuGet\\v3-cache", "Composer\\cache", "go-build"];
    let home = [".gradle\\caches", ".cargo\\registry\\cache", ".bun\\install\\cache"];
    let mut paths: Vec<_> = local.iter().map(|p| u.local_appdata.join(p)).collect();
    paths.extend(home.iter().map(|p| u.profile.join(p)));
    FolderCategory::new(
        "dev",
        "Developer tool caches",
        Text::new("Downloaded npm/pip/yarn/uv/NuGet/Composer/Gradle/Cargo/Bun packages and the Go build cache. They are downloaded again on the next install or build."),
        paths,
    )
}
