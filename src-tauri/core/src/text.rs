use serde::{Deserialize, Serialize};

/// A user-facing text: the English sentence is the key, the UI translates it (src/lang/<code>.json)
/// and fills `{0}`, `{1}`... with the arguments. The core never formats text for a language itself.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<Arg>,
}

/// A text argument. Sizes stay numbers so the UI formats them with the user's number format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Str(String),
    Size { size: f64 },
    Text { text: Box<Text> },
}

impl Text {
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string(), args: Vec::new() }
    }

    pub fn arg(mut self, value: impl ToString) -> Self {
        self.args.push(Arg::Str(value.to_string()));
        self
    }

    pub fn size(mut self, bytes: f64) -> Self {
        self.args.push(Arg::Size { size: bytes });
        self
    }

    pub fn text(mut self, inner: Text) -> Self {
        self.args.push(Arg::Text { text: Box::new(inner) });
        self
    }

    /// English rendering for the log file (sizes in GB/MB/KB like the old program).
    pub fn english(&self) -> String {
        let mut out = self.key.clone();
        for (i, a) in self.args.iter().enumerate() {
            let v = match a {
                Arg::Str(s) => s.clone(),
                Arg::Size { size } => format_size(*size),
                Arg::Text { text } => text.english(),
            };
            out = out.replace(&format!("{{{i}}}"), &v).replace(&format!("{{{i}:N0}}"), &v);
        }
        out
    }
}

/// Size as in the log file: 1.23 GB, 45 MB, 12 KB or 0.
pub fn format_size(b: f64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.0} MB", b / MB)
    } else if b > 0.0 {
        format!("{:.0} KB", b / 1024.0)
    } else {
        "0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_fills_arguments_in_order() {
        let t = Text::new("{0} of {1}").arg("a").size(2.0 * 1024.0 * 1024.0);
        assert_eq!(t.english(), "a of 2 MB");
    }

    #[test]
    fn serializes_size_as_object() {
        let t = Text::new("x {0}").size(5.0);
        assert_eq!(serde_json::to_string(&t).unwrap(), r#"{"key":"x {0}","args":[{"size":5.0}]}"#);
    }
}
