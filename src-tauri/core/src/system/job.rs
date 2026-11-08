//! Job Object with KILL_ON_JOB_CLOSE: the private AI server (and its runner processes) ends together
//! with PC Cleaner, also when the program is closed abruptly.

#[cfg(windows)]
pub struct KillOnCloseJob(windows::Win32::Foundation::HANDLE);

// SAFETY: a job handle may be used and closed from any thread.
#[cfg(windows)]
unsafe impl Send for KillOnCloseJob {}
#[cfg(windows)]
unsafe impl Sync for KillOnCloseJob {}

#[cfg(windows)]
impl KillOnCloseJob {
    pub fn new() -> Option<Self> {
        use windows::Win32::System::JobObjects::*;
        // SAFETY: the info struct is fully initialised and its size passed; the handle is owned by Self.
        unsafe {
            let job = CreateJobObjectW(None, windows::core::PCWSTR::null()).ok()?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let size = std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32;
            SetInformationJobObject(job, JobObjectExtendedLimitInformation, (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(), size).ok()?;
            Some(Self(job))
        }
    }

    pub fn assign(&self, child: &std::process::Child) -> bool {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        // SAFETY: both handles are valid while the call runs.
        unsafe { windows::Win32::System::JobObjects::AssignProcessToJobObject(self.0, HANDLE(child.as_raw_handle())).is_ok() }
    }
}

#[cfg(windows)]
impl Drop for KillOnCloseJob {
    fn drop(&mut self) {
        // SAFETY: closing the last handle kills every process in the job - that is the point.
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}

#[cfg(not(windows))]
pub struct KillOnCloseJob;

#[cfg(not(windows))]
impl KillOnCloseJob {
    pub fn new() -> Option<Self> {
        Some(Self)
    }
    pub fn assign(&self, _child: &std::process::Child) -> bool {
        false
    }
}
