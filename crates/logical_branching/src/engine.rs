use hashbrown::{HashMap, HashSet};
use plugin_response_chain::ActionId;

use crate::error::LogicalBranchError;
use crate::model::{
    BranchDecision, LogicalBranchingConfig, LogicalRule, PluginRuntimeState, PluginStateMatch,
    RuntimeSnapshot,
};

/// Compiled, ready-to-query logical branching engine.
#[derive(Debug, Clone)]
pub struct LogicalBranchingEngine {
    rules: Vec<LogicalRule>,
}

impl LogicalBranchingEngine {
    /// Build engine from config, optionally validating action IDs.
    pub fn from_config(
        cfg: LogicalBranchingConfig,
        allowed_actions: Option<&HashSet<ActionId>>,
    ) -> Result<Self, LogicalBranchError> {
        // Allow empty configs if you want "no constraints".
        // If you prefer strict mode, uncomment:
        // if cfg.rules.is_empty() {
        //     return Err(LogicalBranchError::EmptyConfig);
        // }

        // Validation
        if let Some(allowed) = allowed_actions {
            for (idx, rule) in cfg.rules.iter().enumerate() {
                // 1) Check for contradictory actions
                let mut allowed_set: HashSet<&str> = HashSet::new();
                for a in &rule.allowed_responses {
                    allowed_set.insert(a.as_str());
                }
                for b in &rule.blocked_paths {
                    if allowed_set.contains(b.as_str()) {
                        return Err(LogicalBranchError::ContradictoryAction {
                            action: b.clone(),
                            rule_index: idx,
                        });
                    }
                }

                // 2) Check that all actions referenced exist in allowed_actions
                for a in rule
                    .allowed_responses
                    .iter()
                    .chain(rule.blocked_paths.iter())
                    .chain(rule.fallback.iter())
                {
                    if !allowed.contains(a) {
                        return Err(LogicalBranchError::UnknownAction {
                            action: a.clone(),
                            rule_index: idx,
                        });
                    }
                }
            }
        }

        Ok(Self { rules: cfg.rules })
    }

    /// Returns reference to the internal rule list (primarily for diagnostics).
    pub fn rules(&self) -> &Vec<LogicalRule> {
        &self.rules
    }

    /// Decide which actions are allowed, blocked, or replaced via fallback.
    ///
    /// Semantics:
    /// - First matching rule wins.
    /// - If a rule matches:
    ///   - candidate_actions are stripped of `blocked_paths`.
    ///   - if `allowed_responses` is non-empty, we intersect with that.
    ///   - if result non-empty → that's final.
    ///   - if result empty and fallback non-empty → fallback is used.
    ///   - if result empty and no fallback → everything is effectively blocked.
    /// - If no rule matches, all candidate_actions are allowed unchanged.
    pub fn decide(
        &self,
        snapshot: &RuntimeSnapshot,
        candidate_actions: &[ActionId],
    ) -> BranchDecision {
        if candidate_actions.is_empty() {
            return BranchDecision {
                allowed: Vec::new(),
                blocked: Vec::new(),
                fallback_used: None,
                matched_rule_index: None,
            };
        }

        for (idx, rule) in self.rules.iter().enumerate() {
            if !rule_matches(rule, snapshot) {
                continue;
            }

            // Build a view of current candidates for filtering.
            let mut candidate_set: HashSet<&str> = HashSet::new();
            for a in candidate_actions {
                candidate_set.insert(a.as_str());
            }

            // 1) Apply blacklist.
            let mut blocked: Vec<ActionId> = Vec::new();
            for b in &rule.blocked_paths {
                if candidate_set.remove(b.as_str()) {
                    blocked.push(b.clone());
                }
            }

            // 2) Apply whitelist (if any).
            let mut allowed: Vec<ActionId> = Vec::new();
            if !rule.allowed_responses.is_empty() {
                for a in &rule.allowed_responses {
                    if candidate_set.contains(a.as_str()) {
                        allowed.push(a.clone());
                    }
                }
            } else {
                // No whitelist: whatever remains.
                for a in candidate_actions {
                    if candidate_set.contains(a.as_str()) {
                        allowed.push(a.clone());
                    }
                }
            }

            // 3) Evaluate result.
            if !allowed.is_empty() {
                return BranchDecision {
                    allowed,
                    blocked,
                    fallback_used: None,
                    matched_rule_index: Some(idx),
                };
            }

            // No allowed actions left.
            if !rule.fallback.is_empty() {
                return BranchDecision {
                    allowed: rule.fallback.clone(),
                    blocked: candidate_actions.to_vec(),
                    fallback_used: Some(rule.fallback.clone()),
                    matched_rule_index: Some(idx),
                };
            }

            // Rule matched, but blocked everything and gave no fallback.
            return BranchDecision {
                allowed: Vec::new(),
                blocked: candidate_actions.to_vec(),
                fallback_used: None,
                matched_rule_index: Some(idx),
            };
        }

        // No rule matched: pass through unchanged.
        BranchDecision {
            allowed: candidate_actions.to_vec(),
            blocked: Vec::new(),
            fallback_used: None,
            matched_rule_index: None,
        }
    }
}

fn rule_matches(rule: &LogicalRule, snapshot: &RuntimeSnapshot) -> bool {
    // 1) Context conditions
    for cond in &rule.context_conditions {
        if !snapshot.context_flags.contains(cond) {
            return false;
        }
    }

    // 2) Plugin states
    for (plugin, expected) in &rule.plugin_states {
        let actual = match snapshot.plugin_states.get(plugin) {
            Some(s) => s,
            None => return false,
        };

        if !plugin_state_matches(expected, actual) {
            return false;
        }
    }

    // 3) System modifiers
    for (key, expected_val) in &rule.system_modifiers {
        match snapshot.system_modifiers.get(key) {
            Some(actual_val) if actual_val == expected_val => {}
            _ => return false,
        }
    }

    true
}

fn plugin_state_matches(expected: &PluginStateMatch, actual: &PluginRuntimeState) -> bool {
    match (expected, actual) {
        (PluginStateMatch::Active, PluginRuntimeState::Active) => true,
        (PluginStateMatch::Idle, PluginRuntimeState::Idle) => true,
        (PluginStateMatch::Disabled, PluginRuntimeState::Disabled) => true,
        (PluginStateMatch::Other(e), PluginRuntimeState::Other(a)) => e == a,
        _ => false,
    }
}