use crate::filters::redact::redact_json_in_place;
use crate::ExportError;
use csv::{Writer, WriterBuilder};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs::File;
use std::path::Path;

/// Configuration for CSV export
#[derive(Debug, Clone)]
pub struct CsvConfig {
    /// Include header row
    pub header: bool,
    /// Keys to redact (case-insensitive)
    pub redact_keys: Vec<String>,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            header: true,
            redact_keys: vec![],
        }
    }
}

/// Export a slice of serializable items as CSV.
/// Rows are serialized to JSON objects to perform redaction and stable field ordering.
pub fn export_csv<T: Serialize>(path: &Path, rows: &[T], cfg: &CsvConfig) -> Result<(), ExportError> {
    let file = File::create(path)?;
    let mut wtr: Writer<File> = WriterBuilder::new().has_headers(false).from_writer(file);

    // 1) Serialize rows -> Value and redact
    let mut objs: Vec<Map<String, Value>> = Vec::with_capacity(rows.len());
    let mut keys: BTreeSet<String> = BTreeSet::new(); // stable order

    for row in rows {
        let mut v = serde_json::to_value(row)?;
        redact_json_in_place(&mut v, &cfg.redact_keys);

        let obj = match v {
            Value::Object(m) => m,
            _ => return Err(ExportError::Invalid("CSV row must serialize to a JSON object")),
        };

        for k in obj.keys() {
            keys.insert(k.to_string());
        }
        objs.push(obj);
    }

    // 2) Header
    let ordered: Vec<String> = keys.into_iter().collect();
    if cfg.header {
        wtr.write_record(ordered.iter())?;
    }

    // 3) Rows
    for obj in objs {
        let mut row = Vec::with_capacity(ordered.len());
        for k in &ordered {
            match obj.get(k) {
                Some(Value::Null) | None => row.push(String::new()),
                Some(v) => row.push(serialize_cell(v)),
            }
        }
        wtr.write_record(row)?;
    }

    wtr.flush()?;
    Ok(())
}

fn serialize_cell(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(v).unwrap_or_else(|_| String::new()),
    }
}
