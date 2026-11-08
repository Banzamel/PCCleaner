use crate::scan::folder::FolderCategory;
use crate::text::Text;
use crate::user::UserContext;

pub fn new(u: &UserContext) -> FolderCategory {
    FolderCategory::new(
        "wer",
        "Error reports and memory dumps",
        Text::new("Crash reports, blue screen memory dumps and driver reports (LiveKernelReports, can take several GB). Only needed to diagnose crashes."),
        vec![
            u.local_appdata.join("Microsoft\\Windows\\WER"),
            u.program_data.join("Microsoft\\Windows\\WER\\ReportQueue"),
            u.program_data.join("Microsoft\\Windows\\WER\\ReportArchive"),
            u.local_appdata.join("CrashDumps"),
            u.windir.join("Minidump"),
            u.windir.join("MEMORY.DMP"),
            u.windir.join("LiveKernelReports"),
        ],
    )
}
