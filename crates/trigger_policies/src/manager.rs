use crate::errors::PolicyError;
use crate::model::{PolicySet, TriggerPolicy};
use crate::parser::load_policies;
use crate::evaluator::evaluate;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use chrono::Utc;

static POLICY_CACHE: Lazy<RwLock<Option<PolicySet>>> = Lazy::new(|| RwLock::new(None));

/// Thread-safe load or reload.
pub fn init_policy_cache(path: &str) -> Result<(), PolicyError> {
    let set = load_policies(path)?;
    let mut cache = POLICY_CACHE.write().unwrap();
    *cache = Some(set);
    Ok(())
}

/// Evaluate all policies and return the highest priority match.
pub fn find_match(ctx: &HashMap<String, String>) -> Result<Option<TriggerPolicy>, PolicyError> {
    let cache = POLICY_CACHE.read().unwrap();
    let Some(ref set) = *cache else {
        return Err(PolicyError::Eval("Policy cache uninitialized".into()));
    };
    let mut best: Option<TriggerPolicy> = None;
    for pol in &set.policies {
        if evaluate(pol, ctx)? {
            if let Some(ref cur) = best {
                if pol.priority > cur.priority {
                    best = Some(pol.clone());
                }
            } else {
                best = Some(pol.clone());
            }
        }
    }
    if let Some(ref p) = best {
        log::info!(
            "[{}] Policy matched: {} -> {:?}",
            Utc::now(),
            p.trigger_name,
            p.actions
        );
    }
    Ok(best)
}