use std::path::Path;

/// Folders excluded in Settings: never measured, listed or deleted, also inside a cache folder.
#[derive(Debug, Clone, Default)]
pub struct Excluded {
    prefixes: Vec<String>,
}

/// Lower case, backslashes, exactly one trailing backslash - so prefix checks match whole names.
pub fn normalize(path: &str) -> String {
    let mut s = path.trim().replace('/', "\\").trim_end_matches('\\').to_lowercase();
    s.push('\\');
    s
}

impl Excluded {
    pub fn new<S: AsRef<str>>(folders: &[S]) -> Self {
        let prefixes = folders.iter().map(|f| f.as_ref().trim()).filter(|f| !f.is_empty()).map(normalize).collect();
        Self { prefixes }
    }

    /// The path is an excluded folder or lies inside one.
    pub fn contains(&self, path: &Path) -> bool {
        self.contains_str(&path.to_string_lossy())
    }

    pub fn contains_str(&self, path: &str) -> bool {
        if self.prefixes.is_empty() || path.is_empty() {
            return false;
        }
        let p = normalize(path);
        self.prefixes.iter().any(|e| p.starts_with(e.as_str()))
    }

    /// Deleting the whole folder would also take an excluded folder inside it.
    pub fn holds_excluded(&self, path: &str) -> bool {
        if self.prefixes.is_empty() || path.is_empty() {
            return false;
        }
        let p = normalize(path);
        self.prefixes.iter().any(|e| e.starts_with(p.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_whole_folder_names_only() {
        let e = Excluded::new(&["C:\\Games\\Keep\\"]);
        assert!(e.contains_str("c:\\games\\keep"));
        assert!(e.contains_str("C:\\Games\\Keep\\save.dat"));
        assert!(!e.contains_str("C:\\Games\\Keeper"));
        assert!(e.holds_excluded("C:\\Games"));
        assert!(!e.holds_excluded("C:\\Other"));
    }

    #[test]
    fn empty_list_excludes_nothing() {
        assert!(!Excluded::new::<&str>(&[]).contains_str("C:\\"));
    }
}
