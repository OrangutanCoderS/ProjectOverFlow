use crate::error::PluginChainError;
use crate::model::{ActionId, ChainMap, PluginResponseChain, PluginResponseChainDef, RuleKey};
use hashbrown::{HashMap, HashSet};
use serde_yaml;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Load YAML file into ChainMap (raw on-disk format).
pub fn load_from_file(path: &str) -> Result<ChainMap, PluginChainError> {
    let data = fs::read_to_string(path)?;
    let def: PluginResponseChainDef = serde_yaml::from_str(&data)?;
    Ok(def.chain)
}

/// Compile a raw `PluginResponseChainDef` into a strongly-typed runtime form.
pub fn compile_def(
    def: PluginResponseChainDef,
    allowed_actions: Option<&HashSet<String>>,
) -> Result<PluginResponseChain, PluginChainError> {
    let mut rules: HashMap<RuleKey, Vec<ActionId>> = HashMap::new();

    for (plugin, severity_map) in def.chain.into_iter() {
        if severity_map.is_empty() {
            return Err(PluginChainError::EmptyPlugin(plugin));
        }

        for (severity, pattern_map) in severity_map.into_iter() {
            if pattern_map.is_empty() {
                return Err(PluginChainError::EmptySeverity {
                    plugin: plugin.clone(),
                    severity: severity.clone(),
                });
            }

            for (pattern, actions) in pattern_map.into_iter() {
                // Validate allowed actions
                if let Some(allowed) = allowed_actions {
                    for a in &actions {
                        if !allowed.contains(a) {
                            return Err(PluginChainError::UnknownAction {
                                plugin: plugin.clone(),
                                severity: severity.clone(),
                                pattern: pattern.clone(),
                                action: a.clone(),
                            });
                        }
                    }
                }

                // skip empty action list
                if actions.is_empty() {
                    continue;
                }

                let key = RuleKey::new(&plugin, &severity, &pattern);
                rules.insert(key, actions);
            }
        }
    }

    Ok(PluginResponseChain { rules })
}

/// Load from JSON string.
pub fn load_from_json_str(
    s: &str,
    allowed_actions: Option<&HashSet<String>>,
) -> Result<PluginResponseChain, PluginChainError> {
    let def: PluginResponseChainDef = serde_json::from_str(s)?;
    compile_def(def, allowed_actions)
}

/// Load from any JSON reader.
pub fn load_from_json_reader<R: Read>(
    mut reader: R,
    allowed_actions: Option<&HashSet<String>>,
) -> Result<PluginResponseChain, PluginChainError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    load_from_json_str(&buf, allowed_actions)
}

/// Load JSON file from disk.
pub fn load_from_json_file(
    path: &Path,
    allowed_actions: Option<&HashSet<String>>,
) -> Result<PluginResponseChain, PluginChainError> {
    let data = fs::read_to_string(path)?;
    load_from_json_str(&data, allowed_actions)
}