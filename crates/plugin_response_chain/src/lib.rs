//! Plugin Response Chain – Phase III TriggerGraphs layer
//!
//! plugin -> severity -> pattern -> [action_ids]

pub mod error;
pub mod loader;
pub mod model;

pub use error::PluginChainError;
pub use loader::{
    compile_def, load_from_file, load_from_json_file, load_from_json_reader, load_from_json_str,
};
pub use model::{
    ActionId, ChainMap, PluginResponseChain, PluginResponseChainDef, RuleKey,
};