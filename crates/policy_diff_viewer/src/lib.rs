use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Supported policy formats.
#[derive(Debug, Clone, Copy)]
pub enum PolicyFormat {
    Json,
    Yaml,
    Text,
}

/// Errors from the diff engine.
#[derive(Debug, Error)]
pub enum DiffError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("unsupported or unknown format for {0}")]
    UnsupportedFormat(String),
}

/// What kind of change occurred at a given path.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
}

/// A single diff entry for a logical path in the policy structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffEntry {
    /// JSON-style path, e.g. `rules[3].plugin_chain` or `root.trigger`.
    pub path: String,
    pub kind: DiffKind,
    /// Previous value (if any).
    pub old: Option<JsonValue>,
    /// New value (if any).
    pub new: Option<JsonValue>,
}

/// Convenience wrapper for a pair of files.
#[derive(Debug, Clone)]
pub struct FilePair {
    pub current: PathBuf,
    pub proposed: PathBuf,
    pub format_hint: Option<PolicyFormat>,
}

impl FilePair {
    pub fn new(
        current: impl Into<PathBuf>,
        proposed: impl Into<PathBuf>,
        format_hint: Option<PolicyFormat>,
    ) -> Self {
        Self {
            current: current.into(),
            proposed: proposed.into(),
            format_hint,
        }
    }
}

/// Top-level API: diff two policy files on disk.
pub fn diff_files(
    current: impl AsRef<Path>,
    proposed: impl AsRef<Path>,
    format_hint: Option<PolicyFormat>,
) -> Result<Vec<DiffEntry>, DiffError> {
    let current_path = current.as_ref();
    let proposed_path = proposed.as_ref();

    let current_str = fs::read_to_string(current_path)?;
    let proposed_str = fs::read_to_string(proposed_path)?;

    let effective_format =
        format_hint.unwrap_or_else(|| detect_format_from_extension(current_path, proposed_path));

    match effective_format {
        PolicyFormat::Json => {
            let cur: JsonValue = serde_json::from_str(&current_str)?;
            let new: JsonValue = serde_json::from_str(&proposed_str)?;
            Ok(diff_json_values(&cur, &new))
        }
        PolicyFormat::Yaml => {
            let cur_yaml: serde_yaml::Value = serde_yaml::from_str(&current_str)?;
            let new_yaml: serde_yaml::Value = serde_yaml::from_str(&proposed_str)?;

            let cur: JsonValue = serde_json::to_value(cur_yaml)?;
            let new: JsonValue = serde_json::to_value(new_yaml)?;
            Ok(diff_json_values(&cur, &new))
        }
        PolicyFormat::Text => Ok(diff_text(&current_str, &proposed_str)),
    }
}

/// Detect format from extension if caller didn’t specify.
fn detect_format_from_extension(current: &Path, proposed: &Path) -> PolicyFormat {
    // Prefer explicit extensions; fall back to Text.
    let ext = current
        .extension()
        .or_else(|| proposed.extension())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "json" => PolicyFormat::Json,
        "yaml" | "yml" => PolicyFormat::Yaml,
        _ => PolicyFormat::Text,
    }
}

/// Diff two serde_json::Value trees, returning structural differences.
pub fn diff_json_values(current: &JsonValue, proposed: &JsonValue) -> Vec<DiffEntry> {
    let mut out = Vec::new();
    diff_json_inner("", current, proposed, &mut out);
    out
}

fn diff_json_inner(path: &str, current: &JsonValue, proposed: &JsonValue, out: &mut Vec<DiffEntry>) {
    if current == proposed {
        return;
    }

    match (current, proposed) {
        (JsonValue::Object(cur_map), JsonValue::Object(new_map)) => {
            let mut keys = BTreeSet::new();
            keys.extend(cur_map.keys().cloned());
            keys.extend(new_map.keys().cloned());

            for key in keys {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                let cur_val = cur_map.get(&key);
                let new_val = new_map.get(&key);

                match (cur_val, new_val) {
                    (Some(c), Some(n)) => {
                        diff_json_inner(&child_path, c, n, out);
                    }
                    (Some(c), None) => {
                        out.push(DiffEntry {
                            path: child_path,
                            kind: DiffKind::Removed,
                            old: Some(c.clone()),
                            new: None,
                        });
                    }
                    (None, Some(n)) => {
                        out.push(DiffEntry {
                            path: child_path,
                            kind: DiffKind::Added,
                            old: None,
                            new: Some(n.clone()),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        (JsonValue::Array(cur_arr), JsonValue::Array(new_arr)) => {
            let max_len = std::cmp::max(cur_arr.len(), new_arr.len());

            for idx in 0..max_len {
                let child_path = if path.is_empty() {
                    format!("[{}]", idx)
                } else {
                    format!("{}[{}]", path, idx)
                };

                let cur_val = cur_arr.get(idx);
                let new_val = new_arr.get(idx);

                match (cur_val, new_val) {
                    (Some(c), Some(n)) => {
                        diff_json_inner(&child_path, c, n, out);
                    }
                    (Some(c), None) => {
                        out.push(DiffEntry {
                            path: child_path,
                            kind: DiffKind::Removed,
                            old: Some(c.clone()),
                            new: None,
                        });
                    }
                    (None, Some(n)) => {
                        out.push(DiffEntry {
                            path: child_path,
                            kind: DiffKind::Added,
                            old: None,
                            new: Some(n.clone()),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        // Type change or scalar change → Modified.
        _ => {
            out.push(DiffEntry {
                path: path.to_string(),
                kind: DiffKind::Modified,
                old: Some(current.clone()),
                new: Some(proposed.clone()),
            });
        }
    }
}

/// Simple line-based text diff for things like execution_limiter.py.
pub fn diff_text(current: &str, proposed: &str) -> Vec<DiffEntry> {
    use similar::{ChangeTag, TextDiff};

    let mut entries = Vec::new();
    let diff = TextDiff::from_lines(current, proposed);

    let mut line_current = 0usize;
    let mut line_proposed = 0usize;

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            match change.tag() {
                ChangeTag::Equal => {
                    if change.value().ends_with('\n') {
                        line_current += 1;
                        line_proposed += 1;
                    }
                }

                ChangeTag::Delete => {
                    let path = format!("line:{}", line_current + 1);
                    entries.push(DiffEntry {
                        path,
                        kind: DiffKind::Removed,
                        old: Some(JsonValue::String(change.value().to_string())),
                        new: None,
                    });

                    if change.value().ends_with('\n') {
                        line_current += 1;
                    }
                }

                ChangeTag::Insert => {
                    let path = format!("line:{}", line_proposed + 1);
                    entries.push(DiffEntry {
                        path,
                        kind: DiffKind::Added,
                        old: None,
                        new: Some(JsonValue::String(change.value().to_string())),
                    });

                    if change.value().ends_with('\n') {
                        line_proposed += 1;
                    }
                }
            }
        }
    }

    entries
}

// Re-exports for downstream crates/tests/CLI.
pub use DiffError as PolicyDiffError;
pub use DiffEntry as PolicyDiffEntry;
pub use DiffKind as PolicyDiffKind;
pub use PolicyFormat as PolicyDiffFormat;