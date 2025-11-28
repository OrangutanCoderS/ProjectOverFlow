use crate::errors::ThresholdError;
use crate::model::{ThresholdConfig, ThresholdRule};
use log::info;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::{Arc, RwLock};

static CONFIG: Lazy<Arc<RwLock<Option<ThresholdConfig>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

/// Load thresholds from JSON or YAML
pub fn load_thresholds(path: &str) -> Result<(), ThresholdError> {
    let content = fs::read_to_string(path)?;
    let config: ThresholdConfig = if path.ends_with(".json") {
        serde_json::from_str(&content).map_err(|e| ThresholdError::ConfigParse(e.to_string()))?
    } else {
        serde_yaml::from_str(&content).map_err(|e| ThresholdError::ConfigParse(e.to_string()))?
    };
    *CONFIG.write().unwrap() = Some(config);
    info!("Threshold configuration loaded from {}", path);
    Ok(())
}

/// Evaluate if a metric breaches any threshold
pub fn evaluate(metric: &str, value: f64) -> Result<String, ThresholdError> {
    let cfg = CONFIG.read().unwrap();
    let config = cfg.as_ref().ok_or_else(|| ThresholdError::MissingField("config not loaded".into()))?;

    let rule = match metric {
        "entropy" => &config.entropy,
        "cpu" => &config.cpu,
        "net_spike_kb" => &config.net_spike_kb,
        _ => return Err(ThresholdError::InvalidValue(metric.into())),
    };

    if value < rule.min_safe || value > rule.max_safe {
        return Err(ThresholdError::Evaluation(format!("{} out of safe bounds", metric)));
    }

    let status = if value >= rule.block {
        "BLOCK"
    } else if value >= rule.warn {
        "WARN"
    } else {
        "OK"
    };

    Ok(status.to_string())
}