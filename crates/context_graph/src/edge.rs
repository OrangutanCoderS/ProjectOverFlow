use serde::{Deserialize, Serialize};
use crate::conditions::EdgeConditionSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDef {
    pub to: u16,
    pub weight: f32,
    pub label: Option<String>,
    pub conditions: Option<EdgeConditionSet>,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub to: u16,
    pub weight: f32,
    pub label: Option<String>,
    pub conditions: EdgeConditionSet,
}

impl Edge {
    pub fn validate(&self) -> Result<(), f32> {
        if !self.weight.is_finite() || self.weight <= 0.0 {
            return Err(self.weight);
        }
        Ok(())
    }
}