//! Installs the AI assistant into the program folder on request (Settings): the official Ollama zip
//! from GitHub (no installer, autostart or tray icon) into `ai\ollama`, then the model into `ai\models`.
//! Nothing goes into the user profile or environment variables; uninstalling removes `ai\`.

use std::io::{Read, Write};
use std::path::Path;

use serde::Deserialize;

use super::{client, server::OllamaServer};
use crate::system::{command, wmi};
use crate::text::Text;

const RELEASES: &str = "https://github.com/ollama/ollama/releases/latest/download";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VideoController {
    name: Option<String>,
}

/// Packages for this PC: ARM64 or x64, plus the ROCm libraries when an AMD GPU is present.
pub fn packages(arm: bool, amd_gpu: bool) -> Vec<&'static str> {
    if arm {
        return vec!["ollama-windows-arm64.zip"];
    }
    let mut v = vec!["ollama-windows-amd64.zip"];
    if amd_gpu {
        v.push("ollama-windows-amd64-rocm.zip");
    }
    v
}

fn is_arm() -> bool {
    ["PROCESSOR_ARCHITECTURE", "PROCESSOR_ARCHITEW6432"].iter().any(|v| std::env::var(v).is_ok_and(|a| a.eq_ignore_ascii_case("ARM64")))
}

fn has_amd_gpu() -> bool {
    wmi::query::<VideoController>(wmi::CIMV2, "SELECT Name FROM Win32_VideoController")
        .iter()
        .any(|v| v.name.as_deref().is_some_and(|n| n.contains("AMD") || n.contains("Radeon")))
}

fn download(url: &str, file: &Path, progress: &dyn Fn(Text, f64)) -> Result<(), String> {
    let r = ureq::get(url).call().map_err(|e| e.to_string())?;
    let total: f64 = r.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let mut reader = r.into_reader();
    let mut out = std::fs::File::create(file).map_err(|e| e.to_string())?;
    let (mut buf, mut done) = (vec![0u8; 1 << 20], 0.0);
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as f64;
        if total > 0.0 {
            progress(Text::new("Downloading the AI engine (Ollama, about 1.4 GB)"), done / total);
        }
    }
    Ok(())
}

/// Downloads the engine (when missing) and the model; writes `config.json` with the chosen model.
pub fn install(app_dir: &Path, model: &str, progress: &dyn Fn(Text, f64)) -> Result<(), String> {
    let ai = app_dir.join("ai");
    let engine = ai.join("ollama");
    std::fs::create_dir_all(&engine).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(ai.join("models")).map_err(|e| e.to_string())?;
    if !super::server::exe(&ai).is_file() {
        let zip = ai.join("ollama-download.zip");
        for p in packages(is_arm(), has_amd_gpu()) {
            download(&format!("{RELEASES}/{p}"), &zip, progress)?;
            progress(Text::new("Unpacking..."), 1.0);
            // tar.exe (built into Windows 10 1803+) unpacks zip files
            let r = command::run(&command::system32("tar.exe"), &["-xf", &zip.to_string_lossy(), "-C", &engine.to_string_lossy()]);
            let _ = std::fs::remove_file(&zip);
            if !r.ok() {
                return Err(format!("unpack failed: {p} (tar exit {})", r.code));
            }
        }
        if !super::server::exe(&ai).is_file() {
            return Err("ollama.exe not found in the downloaded archive".into());
        }
    }
    progress(Text::new("Starting the AI engine..."), 0.0);
    let mut server = OllamaServer::default();
    if !server.start(&ai) {
        return Err("Ollama server did not start".into());
    }
    let label = Text::new("Downloading model {0} (about 2.5-5 GB, this can take a while)...").arg(model);
    client::pull(model, &|c, t| progress(label.clone(), if t > 0 { c as f64 / t as f64 } else { 0.0 }))?;
    // The server was only needed to pull the model; PC Cleaner starts it itself, on demand
    server.stop();
    std::fs::write(app_dir.join("config.json"), format!("{{ \"model\": \"{model}\" }}")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_choice() {
        assert_eq!(super::packages(true, true), vec!["ollama-windows-arm64.zip"]);
        assert_eq!(super::packages(false, true).len(), 2);
        assert_eq!(super::packages(false, false), vec!["ollama-windows-amd64.zip"]);
    }
}
