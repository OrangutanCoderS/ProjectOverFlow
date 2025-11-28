use serde_json::Value;

use crate::model::{PolicyChange, PolicyChangeKind, PolicyDiff, PolicyVersion};

/// Compute a structural JSON diff between base and candidate policies.
pub fn diff_policies(
    base_version: Option<PolicyVersion>,
    candidate_version: PolicyVersion,
    base: &Value,
    candidate: &Value,
) -> PolicyDiff {
    let mut changes = Vec::new();
    diff_value("$".to_string(), base, candidate, &mut changes);

    PolicyDiff {
        base_version,
        candidate_version,
        changes,
    }
}

fn diff_value(path: String, base: &Value, candidate: &Value, out: &mut Vec<PolicyChange>) {
    match (base, candidate) {
        (Value::Object(bm), Value::Object(cm)) => {
            // Removed / modified keys
            for (k, bv) in bm {
                let child_path = format!("{}.{}", path, k);
                match cm.get(k) {
                    Some(cv) => diff_value(child_path, bv, cv, out),
                    None => out.push(PolicyChange {
                        path: child_path,
                        kind: PolicyChangeKind::Removed {
                            old_value: bv.clone(),
                        },
                    }),
                }
            }
            // Added keys
            for (k, cv) in cm {
                if !bm.contains_key(k) {
                    let child_path = format!("{}.{}", path, k);
                    out.push(PolicyChange {
                        path: child_path,
                        kind: PolicyChangeKind::Added {
                            new_value: cv.clone(),
                        },
                    });
                }
            }
        }
        (Value::Array(ba), Value::Array(ca)) => {
            let max_len = ba.len().max(ca.len());
            for i in 0..max_len {
                let child_path = format!("{}[{}]", path, i);
                let bv = ba.get(i);
                let cv = ca.get(i);
                match (bv, cv) {
                    (Some(bv), Some(cv)) => diff_value(child_path, bv, cv, out),
                    (Some(bv), None) => out.push(PolicyChange {
                        path: child_path,
                        kind: PolicyChangeKind::Removed {
                            old_value: bv.clone(),
                        },
                    }),
                    (None, Some(cv)) => out.push(PolicyChange {
                        path: child_path,
                        kind: PolicyChangeKind::Added {
                            new_value: cv.clone(),
                        },
                    }),
                    (None, None) => {}
                }
            }
        }
        _ => {
            if base != candidate {
                out.push(PolicyChange {
                    path,
                    kind: PolicyChangeKind::Modified {
                        old_value: base.clone(),
                        new_value: candidate.clone(),
                    },
                });
            }
        }
    }
}