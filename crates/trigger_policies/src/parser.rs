use crate::errors::PolicyError;
use crate::model::PolicySet;
use std::fs::File;
use std::io::Read;

/// Load and parse trigger_policies.yaml safely.
pub fn load_policies(path: &str) -> Result<PolicySet, PolicyError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let parsed: PolicySet = serde_yaml::from_str(&content)?;
    Ok(parsed)
}