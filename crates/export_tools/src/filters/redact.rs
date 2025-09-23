use serde_json::Value;

/// Redact values by key names (case-insensitive). Leaves structure intact.
pub fn redact_json_in_place(v: &mut Value, keys: &[String]) {
    match v {
        Value::Object(map) => {
            for (k, val) in map.iter_mut() {
                if keys.iter().any(|rk| rk.eq_ignore_ascii_case(k)) {
                    *val = Value::String("[REDACTED]".into());
                } else {
                    redact_json_in_place(val, keys);
                }
            }
        }
        Value::Array(arr) => {
            for x in arr {
                redact_json_in_place(x, keys);
            }
        }
        _ => {}
    }
}
