use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde_json;

use crate::error::TelemetryPatternError;
use crate::model::PatternStats;

/// SQLite-backed store for mined patterns.
///
/// This corresponds to `pattern_cache.db` in the blueprint.
pub struct PatternStore {
    conn: Connection,
    path: Option<PathBuf>,
}

impl PatternStore {
    /// Open (or create) an on-disk pattern cache.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TelemetryPatternError> {
        let path_buf = path.as_ref().to_owned();
        let conn = Connection::open(&path_buf)?;
        let mut store = PatternStore {
            conn,
            path: Some(path_buf),
        };
        store.init_schema()?;
        Ok(store)
    }

    /// Open an in-memory pattern cache (useful for tests/benches).
    pub fn open_in_memory() -> Result<Self, TelemetryPatternError> {
        let conn = Connection::open_in_memory()?;
        let mut store = PatternStore { conn, path: None };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&mut self) -> Result<(), TelemetryPatternError> {
        // Simple motifs table; can be extended later with more metrics.
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS motifs (
                fingerprint   TEXT PRIMARY KEY,
                length        INTEGER NOT NULL,
                first_seen    TEXT NOT NULL,
                last_seen     TEXT NOT NULL,
                count         INTEGER NOT NULL,
                avg_entropy   REAL,
                failure_rate  REAL,
                sample_chain  TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    /// Insert or update a pattern.
    ///
    /// If a row with the same fingerprint already exists, we keep the earliest
    /// `first_seen`, latest `last_seen`, and add counts/weighted averages.
    pub fn upsert_pattern(&self, stats: &PatternStats) -> Result<(), TelemetryPatternError> {
        // We do read-modify-write for simple merging.
        if let Some(existing) = self.get_pattern(&stats.fingerprint)? {
            let merged = merge_pattern_stats(&existing, stats)?;
            self.insert_or_replace(&merged)
        } else {
            self.insert_or_replace(stats)
        }
    }

    fn insert_or_replace(&self, stats: &PatternStats) -> Result<(), TelemetryPatternError> {
        let first_seen = stats.first_seen.to_rfc3339();
        let last_seen = stats.last_seen.to_rfc3339();
        let sample_chain_json = serde_json::to_string(&stats.sample_chain)?;

        self.conn.execute(
            r#"
            INSERT INTO motifs (
                fingerprint, length, first_seen, last_seen,
                count, avg_entropy, failure_rate, sample_chain
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(fingerprint) DO UPDATE SET
                length       = excluded.length,
                first_seen   = excluded.first_seen,
                last_seen    = excluded.last_seen,
                count        = excluded.count,
                avg_entropy  = excluded.avg_entropy,
                failure_rate = excluded.failure_rate,
                sample_chain = excluded.sample_chain;
            "#,
            params![
                stats.fingerprint,
                stats.length as i64,
                first_seen,
                last_seen,
                stats.count as i64,
                stats.avg_entropy,
                stats.failure_rate,
                sample_chain_json,
            ],
        )?;
        Ok(())
    }

    /// Fetch a single pattern by fingerprint, if present.
    pub fn get_pattern(
        &self,
        fingerprint: &str,
    ) -> Result<Option<PatternStats>, TelemetryPatternError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT fingerprint, length, first_seen, last_seen,
                   count, avg_entropy, failure_rate, sample_chain
            FROM motifs
            WHERE fingerprint = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![fingerprint])?;
        if let Some(row) = rows.next()? {
            let stats = row_to_pattern_stats(row)?;
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }

    /// Return the top-N most frequent patterns by `count`.
    pub fn top_patterns(
        &self,
        limit: usize,
    ) -> Result<Vec<PatternStats>, TelemetryPatternError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT fingerprint, length, first_seen, last_seen,
                   count, avg_entropy, failure_rate, sample_chain
            FROM motifs
            ORDER BY count DESC, last_seen DESC
            LIMIT ?1
            "#,
        )?;

        let mut rows = stmt.query(params![limit as i64])?;
        let mut out = Vec::new();

        while let Some(row) = rows.next()? {
            out.push(row_to_pattern_stats(row)?);
        }

        Ok(out)
    }

    /// Return the underlying DB path, if on-disk.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

fn row_to_pattern_stats(
    row: &rusqlite::Row<'_>,
) -> Result<PatternStats, TelemetryPatternError> {
    let fingerprint: String = row.get(0)?;
    let length: i64 = row.get(1)?;
    let first_seen_str: String = row.get(2)?;
    let last_seen_str: String = row.get(3)?;
    let count: i64 = row.get(4)?;
    let avg_entropy: Option<f64> = row.get(5)?;
    let failure_rate: Option<f64> = row.get(6)?;
    let sample_chain_json: String = row.get(7)?;

    let first_seen = parse_rfc3339(&first_seen_str)?;
    let last_seen = parse_rfc3339(&last_seen_str)?;
    let sample_chain: Vec<String> = serde_json::from_str(&sample_chain_json)?;

    Ok(PatternStats {
        fingerprint,
        length: length as u32,
        first_seen,
        last_seen,
        count: count as u64,
        avg_entropy,
        failure_rate,
        sample_chain,
    })
}

fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, TelemetryPatternError> {
    let dt = chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|e| TelemetryPatternError::Time(e.to_string()))?;
    Ok(dt.with_timezone(&Utc))
}

fn merge_pattern_stats(
    a: &PatternStats,
    b: &PatternStats,
) -> Result<PatternStats, TelemetryPatternError> {
    if a.fingerprint != b.fingerprint {
        return Err(TelemetryPatternError::Inconsistent(
            "fingerprint mismatch in merge".to_string(),
        ));
    }

    let first_seen = if a.first_seen <= b.first_seen {
        a.first_seen
    } else {
        b.first_seen
    };

    let last_seen = if a.last_seen >= b.last_seen {
        a.last_seen
    } else {
        b.last_seen
    };

    let count = a.count + b.count;

    let avg_entropy = match (a.avg_entropy, b.avg_entropy) {
    (Some(x), None) => Some(x),
    (None, Some(y)) => Some(y),
    (Some(x), Some(y)) => Some((x + y) / 2.0),
    (None, None) => None,
};

    let failure_rate = match (a.failure_rate, b.failure_rate) {
    (Some(x), None) => Some(x),
    (None, Some(y)) => Some(y),
    (Some(x), Some(y)) => Some((x + y) / 2.0),
    (None, None) => None,
};

    Ok(PatternStats {
        fingerprint: a.fingerprint.clone(),
        length: a.length.max(b.length),
        first_seen,
        last_seen,
        count,
        avg_entropy,
        failure_rate,
        // prefer the longer sample chain if lengths differ
        sample_chain: if a.sample_chain.len() >= b.sample_chain.len() {
            a.sample_chain.clone()
        } else {
            b.sample_chain.clone()
        },
    })
}