use std::io;

use serde_json;
use serde_yaml;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// High-level configuration / planning errors for the execution node.
#[derive(Debug, Error)]
pub enum ExecutionNodeError {
    #[error("I/O error while loading execution config: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error in execution config: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML parse error in execution config: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("missing policy template for policy id `{0}`")]
    MissingPolicyTemplate(String),

    #[error(
        "step limit exceeded for policy `{policy_id}` (attempted {attempted}, max {max})"
    )]
    StepLimitExceeded {
        policy_id: String,
        attempted: usize,
        max: usize,
    },

    #[error("invalid execution config: {0}")]
    InvalidConfig(String),
}

/// Error codes for individual step execution failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecErrorCode {
    BackendFailure,
    InvalidTarget,
    InvalidParams,
    InternalError,
}

/// Concrete execution error returned by backends.
#[derive(Debug, Error)]
#[error("{code:?}: {message}")]
pub struct ExecError {
    pub code: ExecErrorCode,
    pub message: String,
}

impl ExecError {
    pub fn backend_failure(msg: impl Into<String>) -> Self {
        Self {
            code: ExecErrorCode::BackendFailure,
            message: msg.into(),
        }
    }

    pub fn invalid_target(msg: impl Into<String>) -> Self {
        Self {
            code: ExecErrorCode::InvalidTarget,
            message: msg.into(),
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: ExecErrorCode::InvalidParams,
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            code: ExecErrorCode::InternalError,
            message: msg.into(),
        }
    }
}
