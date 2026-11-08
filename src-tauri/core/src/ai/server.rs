use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

use super::{client, HOST};
use crate::system::command;
use crate::system::job::KillOnCloseJob;

/// The private Ollama server. Dropping it ends the server and its runner processes (Job Object).
#[derive(Default)]
pub struct OllamaServer {
    job: Option<KillOnCloseJob>,
    child: Option<Child>,
}

pub fn exe(ai_dir: &Path) -> PathBuf {
    ai_dir.join("ollama").join("ollama.exe")
}

impl OllamaServer {
    /// Starts `ai\ollama\ollama.exe serve` unless it already answers; waits up to 20 s.
    /// Models and keys stay in `ai\` - nothing goes to the user profile.
    pub fn start(&mut self, ai_dir: &Path) -> bool {
        if client::ping() {
            return true;
        }
        let exe = exe(ai_dir);
        if !exe.is_file() {
            return false;
        }
        // Ollama keeps its keys in the home folder and may write caches to (Local)AppData: all of it
        // is pointed inside ai\, so nothing lands in the user profile
        let appdata = ai_dir.join("appdata");
        let _ = std::fs::create_dir_all(&appdata);
        let child = command::hidden(&exe.to_string_lossy())
            .arg("serve")
            .env("OLLAMA_HOST", HOST)
            .env("OLLAMA_MODELS", ai_dir.join("models"))
            .env("USERPROFILE", ai_dir)
            .env("HOME", ai_dir)
            .env("APPDATA", &appdata)
            .env("LOCALAPPDATA", &appdata)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        let Ok(child) = child else { return false };
        let job = KillOnCloseJob::new();
        if let Some(j) = &job {
            j.assign(&child);
        }
        self.job = job;
        self.child = Some(child);
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(500));
            if client::ping() {
                return true;
            }
        }
        false
    }

    /// Ends the server (frees VRAM and RAM, e.g. before cleaning).
    pub fn stop(&mut self) {
        self.job = None;
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for OllamaServer {
    fn drop(&mut self) {
        self.stop();
    }
}
