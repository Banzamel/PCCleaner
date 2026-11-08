use std::fs::Metadata;

/// FILE_ATTRIBUTE_REPARSE_POINT: junctions, symbolic links, OneDrive placeholders and other links.
pub const REPARSE_POINT: u32 = 0x400;
pub const READONLY: u32 = 0x1;
/// Cloud-only files (OneDrive): RECALL_ON_DATA_ACCESS, RECALL_ON_OPEN, OFFLINE - reading them downloads them.
pub const CLOUD_ONLY: u32 = 0x40_0000 | 0x4_0000 | 0x1000;

/// Raw Windows attributes (0 on other systems).
pub fn attributes(md: &Metadata) -> u32 {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        md.file_attributes()
    }
    #[cfg(not(windows))]
    {
        let _ = md;
        0
    }
}

/// A link of any kind, taken from metadata that was read WITHOUT following links
/// (`symlink_metadata` or `DirEntry::metadata`).
pub fn is_link(md: &Metadata) -> bool {
    md.file_type().is_symlink() || attributes(md) & REPARSE_POINT != 0
}
