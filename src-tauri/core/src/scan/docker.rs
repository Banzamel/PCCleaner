//! Docker disk usage from `docker system df` (only while Docker runs).

use crate::system::command;

/// Space per type: in use (SIZE) and reclaimable (RECLAIMABLE).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parts {
    pub images: f64,
    pub containers: f64,
    pub volumes: f64,
    pub cache: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockerUsage {
    pub running: bool,
    pub reclaimable: Parts,
    pub used: Parts,
    pub in_use: f64,
}

impl DockerUsage {
    pub fn reclaimable_total(&self) -> f64 {
        let r = &self.reclaimable;
        r.images + r.containers + r.volumes + r.cache
    }
}

/// "1.234GB", "512kB (40%)", "0B" -> bytes (decimal units, as Docker prints them).
pub fn parse_size(s: &str) -> f64 {
    let s = s.trim();
    let num: String = s.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    let rest = s[num.len()..].trim_start();
    let Ok(n) = num.parse::<f64>() else { return 0.0 };
    let mult = match rest.chars().next() {
        Some('k' | 'K') => 1e3,
        Some('M') => 1e6,
        Some('G') => 1e9,
        Some('T') => 1e12,
        Some('B') => 1.0,
        _ => return 0.0,
    };
    n * mult
}

/// Parses the `--format '{{json .}}'` lines.
pub fn parse_df(lines: &[&str]) -> DockerUsage {
    let mut u = DockerUsage { running: !lines.is_empty(), ..Default::default() };
    for l in lines {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        let field = |k: &str| parse_size(v.get(k).and_then(|x| x.as_str()).unwrap_or(""));
        let (reclaim, size) = (field("Reclaimable"), field("Size"));
        u.in_use += size;
        let ty = v.get("Type").and_then(|x| x.as_str()).unwrap_or("");
        let slot = match ty {
            "Images" => Some((&mut u.reclaimable.images, &mut u.used.images)),
            "Containers" => Some((&mut u.reclaimable.containers, &mut u.used.containers)),
            "Build Cache" => Some((&mut u.reclaimable.cache, &mut u.used.cache)),
            t if t.ends_with("Volumes") => Some((&mut u.reclaimable.volumes, &mut u.used.volumes)),
            _ => None,
        };
        if let Some((r, s)) = slot {
            *r = reclaim;
            *s = size;
        }
    }
    u
}

pub fn usage() -> DockerUsage {
    let out = command::run("docker", &["system", "df", "--format", "{{json .}}"]);
    if !out.ok() {
        return DockerUsage::default();
    }
    parse_df(&out.lines().collect::<Vec<_>>())
}

/// Runs a docker command and returns its output lines.
pub fn docker(args: &[&str]) -> Vec<String> {
    command::run("docker", args).lines().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(parse_size("1.5GB"), 1.5e9);
        assert_eq!(parse_size("512kB (40%)"), 512e3);
        assert_eq!(parse_size("0B"), 0.0);
        assert_eq!(parse_size("junk"), 0.0);
    }

    #[test]
    fn df_lines() {
        let lines = [
            r#"{"Type":"Images","Size":"2GB","Reclaimable":"1GB (50%)"}"#,
            r#"{"Type":"Local Volumes","Size":"3GB","Reclaimable":"0B"}"#,
            r#"{"Type":"Build Cache","Size":"500MB","Reclaimable":"500MB"}"#,
        ];
        let u = parse_df(&lines);
        assert!(u.running);
        assert_eq!(u.in_use, 5.5e9);
        assert_eq!(u.reclaimable.cache, 5e8);
        assert_eq!(u.used.volumes, 3e9);
        assert_eq!(u.reclaimable_total(), 1.5e9);
    }
}
