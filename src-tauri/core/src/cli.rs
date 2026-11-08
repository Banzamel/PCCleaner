/// Command-line parameters. The old names (`-Preview`, Polish aliases such as `-Podglad`) keep
/// working, so existing shortcuts and the automatic cleaning task stay valid.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CliArgs {
    pub preview: bool,
    pub no_ai: bool,
    pub no_update_check: bool,
    pub auto_clean: bool,
    pub model: Option<String>,
    pub drives: Vec<String>,
    pub language: Option<String>,
    /// Account that started the program (passed on by the elevation).
    pub sid: Option<String>,
}

fn flag_name(arg: &str) -> Option<String> {
    let name = arg.strip_prefix("--").or_else(|| arg.strip_prefix('-')).or_else(|| arg.strip_prefix('/'))?;
    Some(name.to_ascii_lowercase())
}

fn drive_letters(v: &str) -> impl Iterator<Item = String> + '_ {
    v.split(',').map(|d| d.trim().trim_end_matches(['\\', ':']).to_ascii_uppercase()).filter(|d| !d.is_empty())
}

/// Parses the arguments after the program name. Unknown arguments are ignored.
pub fn parse<I: IntoIterator<Item = String>>(args: I) -> CliArgs {
    let mut a = CliArgs::default();
    let mut it = args.into_iter().peekable();
    while let Some(arg) = it.next() {
        let Some(name) = flag_name(&arg) else { continue };
        match name.as_str() {
            "preview" | "podglad" => a.preview = true,
            "noai" | "no-ai" | "bezllm" => a.no_ai = true,
            "noupdatecheck" | "no-update-check" | "bezaktualizacji" => a.no_update_check = true,
            "autoclean" | "auto-clean" => a.auto_clean = true,
            "model" => a.model = it.next_if(|v| flag_name(v).is_none()).filter(|v| !v.trim().is_empty()),
            "language" | "jezyk" => {
                a.language = it
                    .next_if(|v| flag_name(v).is_none())
                    .map(|v| v.to_ascii_lowercase())
                    .filter(|v| matches!(v.as_str(), "pl" | "en" | "es"))
            }
            "sid" => a.sid = it.next_if(|v| flag_name(v).is_none()),
            "drives" | "dyski" => {
                while let Some(v) = it.next_if(|v| flag_name(v).is_none()) {
                    a.drives.extend(drive_letters(&v));
                }
            }
            _ => {}
        }
    }
    a
}

/// Arguments for the elevated copy of the program: the same choices plus the starting account.
pub fn to_args(a: &CliArgs, sid: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for (on, flag) in [(a.preview, "--preview"), (a.no_ai, "--no-ai"), (a.no_update_check, "--no-update-check"), (a.auto_clean, "--auto-clean")] {
        if on {
            v.push(flag.into());
        }
    }
    if let Some(m) = &a.model {
        v.extend(["--model".to_string(), m.clone()]);
    }
    if let Some(l) = &a.language {
        v.extend(["--language".to_string(), l.clone()]);
    }
    if !a.drives.is_empty() {
        v.extend(["--drives".to_string(), a.drives.join(",")]);
    }
    v.extend(["--sid".to_string(), sid.to_string()]);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> CliArgs {
        parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn old_and_new_names() {
        let a = p("-Podglad -BezLLM --model qwen3:8b -Dyski C,p: -Jezyk ES -AutoClean");
        assert!(a.preview && a.no_ai && a.auto_clean);
        assert_eq!(a.model.as_deref(), Some("qwen3:8b"));
        assert_eq!(a.drives, vec!["C", "P"]);
        assert_eq!(a.language.as_deref(), Some("es"));
    }

    #[test]
    fn round_trip_through_elevation() {
        let a = p("--preview --drives D --language pl");
        let back = parse(to_args(&a, "S-1-5-21-1"));
        assert_eq!(back.sid.as_deref(), Some("S-1-5-21-1"));
        assert_eq!(back.drives, vec!["D"]);
        assert!(back.preview);
    }

    #[test]
    fn invalid_language_is_ignored() {
        assert_eq!(p("-Language de").language, None);
    }
}
