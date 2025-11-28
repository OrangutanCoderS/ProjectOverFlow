use crate::model::Pattern;

/// Query utilities for in-memory filtering & ranking
pub struct PatternQuery;

impl PatternQuery {
    pub fn top_k(mut patterns: Vec<Pattern>, k: usize) -> Vec<Pattern> {
        patterns.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        patterns.into_iter().take(k).collect()
    }

    pub fn filter_by_score(patterns: &[Pattern], min_score: f64) -> Vec<Pattern> {
        patterns
            .iter()
            .cloned()
            .filter(|p| p.score >= min_score)
            .collect()
    }
}
