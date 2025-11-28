use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Raw on-disk definition that matches plugin_response_chain.json.
///
/// Shape:

/// Plugin → Event → Action → Vec<NextPlugin>
pub type ChainMap = HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponseChainDef {
    pub chain: ChainMap,
}

/// Type alias for action identifier strings from the config.
pub type ActionId = String;

/// Key used internally for fast lookup inside the compiled map:
/// (plugin, severity, pattern)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuleKey {
    pub plugin: String,
    pub severity: String,
    pub pattern: String,
}

impl RuleKey {
    pub fn new<P, S, T>(plugin: P, severity: S, pattern: T) -> Self
    where
        P: Into<String>,
        S: Into<String>,
        T: Into<String>,
    {
        Self {
            plugin: plugin.into(),
            severity: severity.into(),
            pattern: pattern.into(),
        }
    }
}

/// In-memory compiled representation for O(1) lookups.
#[derive(Debug, Clone)]
pub struct PluginResponseChain {
    /// (plugin, severity, pattern) -> ordered list of action IDs
    pub(crate) rules: HashMap<RuleKey, Vec<ActionId>>,
}

impl PluginResponseChain {
    /// Resolve a set of actions for a given (plugin, severity, pattern) triple.
    ///
    /// Returns `None` if there is no rule for this combination.
    pub fn resolve<'a>(
        &'a self,
        plugin: &str,
        severity: &str,
        pattern: &str,
    ) -> Option<&'a [ActionId]> {
        let key = RuleKey {
            plugin: plugin.to_string(),
            severity: severity.to_string(),
            pattern: pattern.to_string(),
        };
        self.rules.get(&key).map(|v| v.as_slice())
    }

    /// Total number of unique (plugin, severity, pattern) rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}