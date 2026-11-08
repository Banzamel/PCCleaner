use std::path::Path;

use super::DEFAULT_MODEL;
use crate::storage::json::read_object;

/// Model name from a manifest path relative to `ai\models\manifests`:
/// `registry.ollama.ai\library\qwen3\4b-instruct` -> `qwen3:4b-instruct`.
pub fn model_from_manifest(parts: &[&str]) -> Option<String> {
    match parts {
        [_, "library", name, tag] => Some(format!("{name}:{tag}")),
        [_, ns, name, tag] => Some(format!("{ns}/{name}:{tag}")),
        _ => None,
    }
}

/// Models downloaded to `ai\models`.
pub fn local_models(ai_dir: &Path) -> Vec<String> {
    let root = ai_dir.join("models").join("manifests");
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(rel) = p.strip_prefix(&root) {
                let parts: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
                let refs: Vec<&str> = parts.iter().map(String::as_str).collect();
                out.extend(model_from_manifest(&refs));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The model chosen when the AI was installed (`config.json` next to the program), or the default.
pub fn installer_model(app_dir: &Path) -> String {
    read_object(&app_dir.join("config.json"))
        .and_then(|m| m.get("model").and_then(|v| v.as_str()).map(String::from))
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_names() {
        assert_eq!(model_from_manifest(&["registry.ollama.ai", "library", "qwen3", "8b"]).as_deref(), Some("qwen3:8b"));
        assert_eq!(model_from_manifest(&["registry.ollama.ai", "user", "m", "latest"]).as_deref(), Some("user/m:latest"));
        assert_eq!(model_from_manifest(&["x"]), None);
    }

    #[test]
    fn reads_local_models_and_config() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("models/manifests/registry.ollama.ai/library/qwen3");
        std::fs::create_dir_all(&m).unwrap();
        std::fs::write(m.join("4b-instruct"), "{}").unwrap();
        assert_eq!(local_models(d.path()), vec!["qwen3:4b-instruct"]);
        assert_eq!(installer_model(d.path()), DEFAULT_MODEL);
        std::fs::write(d.path().join("config.json"), "{ \"model\": \"qwen3:8b\" }").unwrap();
        assert_eq!(installer_model(d.path()), "qwen3:8b");
    }
}
