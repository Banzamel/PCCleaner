//! Stopping and starting Windows services (Windows Update and BITS around their cache).

#[cfg(windows)]
mod imp {
    use std::time::{Duration, Instant};
    use windows::core::PCWSTR;
    use windows::Win32::System::Services::*;

    struct Handle(SC_HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            // SAFETY: the handle came from OpenSCManagerW/OpenServiceW and is closed once.
            let _ = unsafe { CloseServiceHandle(self.0) };
        }
    }

    fn open(name: &str, access: u32) -> Option<(Handle, Handle)> {
        let w = crate::system::wide(name);
        // SAFETY: valid NUL-terminated name; handles are wrapped so they are always closed.
        unsafe {
            let scm = Handle(OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT).ok()?);
            let svc = Handle(OpenServiceW(scm.0, PCWSTR(w.as_ptr()), access).ok()?);
            Some((scm, svc))
        }
    }

    fn state(svc: &Handle) -> Option<SERVICE_STATUS_CURRENT_STATE> {
        let mut st = SERVICE_STATUS::default();
        // SAFETY: valid service handle and status struct.
        unsafe { QueryServiceStatus(svc.0, &mut st).ok()? };
        Some(st.dwCurrentState)
    }

    /// Stops a service and waits (up to 30 s) until it has stopped, like Stop-Service -Force.
    pub fn stop(name: &str) {
        let Some((_scm, svc)) = open(name, SERVICE_STOP | SERVICE_QUERY_STATUS) else { return };
        let mut st = SERVICE_STATUS::default();
        // SAFETY: valid handle; a service that is already stopped just returns an error.
        let _ = unsafe { ControlService(svc.0, SERVICE_CONTROL_STOP, &mut st) };
        let until = Instant::now() + Duration::from_secs(30);
        while Instant::now() < until && state(&svc) != Some(SERVICE_STOPPED) {
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    pub fn start(name: &str) {
        let Some((_scm, svc)) = open(name, SERVICE_START) else { return };
        // SAFETY: valid handle, no start arguments.
        let _ = unsafe { StartServiceW(svc.0, None) };
    }
}

#[cfg(windows)]
pub use imp::{start, stop};

#[cfg(not(windows))]
pub fn stop(_name: &str) {}
#[cfg(not(windows))]
pub fn start(_name: &str) {}
