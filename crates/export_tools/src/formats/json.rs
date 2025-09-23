use crate::filters::redact::redact_json_in_place;
use crate::ExportError;
use serde::Serialize;
use serde_json::{to_writer_pretty, Value};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Configuration for JSON export
#[derive(Debug, Clone)]
pub struct JsonConfig {
    /// Pretty print output (true) vs compact (false)
    pub pretty: bool,
    /// Keys to redact (case-insensitive)
    pub redact_keys: Vec<String>,
}

impl Default for JsonConfig {
    fn default() -> Self {
        Self {
            pretty: true,
            redact_keys: vec![],
        }
    }
}

/// Export a slice of serializable items to JSON (pretty or compact).
/// Items are serialized to Value first so we can redact safely.
pub fn export_json<T: Serialize>(path: &Path, rows: &[T], cfg: &JsonConfig) -> Result<(), ExportError> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);

    if cfg.pretty {
        // Pretty array write: redact per element
        let mut redacted: Vec<Value> = Vec::with_capacity(rows.len());
        for row in rows {
            let mut v = serde_json::to_value(row)?;
            redact_json_in_place(&mut v, &cfg.redact_keys);
            redacted.push(v);
        }
        to_writer_pretty(&mut w, &redacted)?;
        writeln!(&mut w)?; // newline for POSIX tools
    } else {
        // Compact array write: stream by hand to avoid buffering entire Vec<Value>
        write!(&mut w, "[")?;
        let mut first = true;
        for row in rows {
            let mut v = serde_json::to_value(row)?;
            redact_json_in_place(&mut v, &cfg.redact_keys);
            if !first { write!(&mut w, ",")?; } else { first = false; }
            serde_json::to_writer(&mut w, &v)?;
        }
        writeln!(&mut w, "]")?;
    }

    w.flush()?;
    Ok(())
}
