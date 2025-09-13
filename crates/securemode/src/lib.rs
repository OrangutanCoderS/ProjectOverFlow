/*!
 Module 19 — Secure Mode Controller
 Goal: Escalate security posture based on telemetry.
 */

use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};
use chrono::Utc;
use thiserror::Error;

// === Data Models === //

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecureLevel {
    Normal,
    Elevated,
    Contained,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecureEvent {
    pub timestamp: String,
    pub level: SecureLevel,
    pub reason: String,
    pub suspicion_score: f64,
    pub anomaly_count: u32,
}

impl SecureEvent {
    pub fn validate(&self) -> Result<()> {
        if self.reason.trim().is_empty() {
            bail!("reason empty");
        }
        if !(0.0..=1.0).contains(&self.suspicion_score) {
            bail!("invalid suspicion_score");
        }
        Ok(())
    }
}

// === Config === //

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    pub elevated_threshold: f64,
    pub contained_threshold: f64,
    pub locked_threshold: f64,
    pub anomaly_threshold: u32,
}

impl Default for SecureConfig {
    fn default() -> Self {
        Self {
            elevated_threshold: 0.3,
            contained_threshold: 0.6,
            locked_threshold: 0.85,
            anomaly_threshold: 5,
        }
    }
}

// === Core Evaluation === //

pub fn evaluate_secure_mode(cfg: &SecureConfig, score: f64, anomalies: u32, reason: &str) -> Result<SecureEvent> {
    let level = if score >= cfg.locked_threshold || anomalies > cfg.anomaly_threshold * 2 {
        SecureLevel::Locked
    } else if score >= cfg.contained_threshold || anomalies > cfg.anomaly_threshold {
        SecureLevel::Contained
    } else if score >= cfg.elevated_threshold {
        SecureLevel::Elevated
    } else {
        SecureLevel::Normal
    };

    let event = SecureEvent {
        timestamp: Utc::now().to_rfc3339(),
        level,
        reason: reason.to_string(),
        suspicion_score: score.clamp(0.0, 1.0),
        anomaly_count: anomalies,
    };
    event.validate()?;
    Ok(event)
}

// === Future Action Hooks (stubs only) === //

#[allow(dead_code)]
fn apply_containment() {
    // Stub for containment measures (network quarantine, etc.)
}

#[allow(dead_code)]
fn lockdown_system() {
    // Stub for full system lockdown
}

// === Error Surface === //

#[derive(Debug, Error)]
pub enum SecureModeError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}
