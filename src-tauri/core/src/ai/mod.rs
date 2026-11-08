//! The local AI assistant: a private Ollama server in the program folder (`ai\`), on its own port so
//! it never clashes with an Ollama the user installed. It runs only while PC Cleaner is open and only
//! after the user asks for something. All traffic stays on 127.0.0.1.

pub mod client;
pub mod install;
pub mod models;
pub mod server;

/// Private port, the same one the PowerShell version used.
pub const HOST: &str = "127.0.0.1:11435";
pub const DEFAULT_MODEL: &str = "qwen3:4b-instruct";

pub fn base_url() -> String {
    format!("http://{HOST}")
}
