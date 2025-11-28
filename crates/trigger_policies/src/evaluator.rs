use crate::errors::PolicyError;
use crate::model::TriggerPolicy;
use regex::Regex;
use std::collections::HashMap;

/// Evaluate a simple key-value context against a policy’s conditions.
pub fn evaluate(policy: &TriggerPolicy, ctx: &HashMap<String, String>) -> Result<bool, PolicyError> {
    for (key, expr) in &policy.conditions {
        match ctx.get(key) {
            Some(val) => {
                if !match_condition(expr, val)? {
                    return Ok(false);
                }
            }
            None => return Err(PolicyError::Eval(format!("Missing key {key}"))),
        }
    }
    Ok(true)
}

/// Core operator parser (>,<,=,regex:)
fn match_condition(expr: &str, val: &str) -> Result<bool, PolicyError> {
    if let Some(num_str) = expr.strip_prefix('>') {
        return Ok(val.parse::<f64>().unwrap_or(0.0) > num_str.parse::<f64>().unwrap_or(0.0));
    }
    if let Some(num_str) = expr.strip_prefix('<') {
        return Ok(val.parse::<f64>().unwrap_or(0.0) < num_str.parse::<f64>().unwrap_or(0.0));
    }
    if let Some(rgx) = expr.strip_prefix("regex:") {
        let re = Regex::new(rgx).map_err(|e| PolicyError::Eval(e.to_string()))?;
        return Ok(re.is_match(val));
    }
    Ok(val == expr)
}