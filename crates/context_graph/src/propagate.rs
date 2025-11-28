use crate::errors::ContextGraphError;

#[derive(Debug, Clone)]
pub struct PropagationConfig {
    pub max_depth: u8,
    pub decay_per_hop: f32,
    pub min_score: f32,
}

pub fn propagate_score(
    initial: f32,
    weight: f32,
    depth: u8,
    cfg: &PropagationConfig,
) -> Result<f32, ContextGraphError> {
    if depth > cfg.max_depth {
        return Err(ContextGraphError::DepthExceeded);
    }

    let mut score = initial * weight * cfg.decay_per_hop.powi(depth as i32);

    if !score.is_finite() {
        score = 0.0;
    }

    if score < cfg.min_score {
        score = 0.0;
    }

    Ok(score)
}