/// The Windows display language when it is Polish or Spanish, English otherwise.
#[cfg(windows)]
pub fn ui_language() -> &'static str {
    // SAFETY: plain query of the user's UI language id.
    let id = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
    language_of(id)
}

#[cfg(not(windows))]
pub fn ui_language() -> &'static str {
    "en"
}

/// Primary language of a LANGID: 0x15 Polish, 0x0A Spanish.
pub fn language_of(langid: u16) -> &'static str {
    match langid & 0x3FF {
        0x15 => "pl",
        0x0A => "es",
        _ => "en",
    }
}

/// Own taskbar identity: the same id as the installer's shortcuts, so pinning and notifications
/// belong to PC Cleaner.
#[cfg(windows)]
pub fn set_app_user_model_id(id: &str) {
    let w = super::wide(id);
    // SAFETY: NUL-terminated id; the call only stores it for this process.
    let _ = unsafe { windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(windows::core::PCWSTR(w.as_ptr())) };
}

#[cfg(not(windows))]
pub fn set_app_user_model_id(_id: &str) {}

#[cfg(test)]
mod tests {
    #[test]
    fn languages() {
        assert_eq!(super::language_of(0x0415), "pl");
        assert_eq!(super::language_of(0x0C0A), "es");
        assert_eq!(super::language_of(0x0409), "en");
    }
}
