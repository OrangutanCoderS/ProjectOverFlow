//! trigger_policies — safe declarative trigger policy engine.
pub mod errors;
pub mod model;
pub mod parser;
pub mod evaluator;
pub mod manager;

pub use errors::PolicyError;
pub use model::{TriggerPolicy, PolicySet};
pub use parser::load_policies;
pub use evaluator::evaluate;
pub use manager::{init_policy_cache, find_match};