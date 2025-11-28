use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;

use crate::error::PatternCacheError;
use crate::model::{NewPattern, Pattern};
use crate::schema::INIT_SQL;

/// The Pattern Cache Engine — lightweight ACID SQLite wrapper
pub struct PatternCacheEngine {
    conn: Arc<Mutex<Connection>>,
}

impl PatternCacheEngine {
    /// Create new cache at path
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, PatternCacheError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(INIT_SQL)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a new pattern into DB
    pub fn insert(&self, p: &NewPattern) -> Result<i64, PatternCacheError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO patterns (category, fingerprint, score, first_seen, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                p.category,
                p.fingerprint,
                p.score,
                p.first_seen.to_rfc3339(),
                p.last_seen.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Retrieve all patterns of a category
    pub fn fetch_by_category(&self, category: &str) -> Result<Vec<Pattern>, PatternCacheError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, category, fingerprint, score, first_seen, last_seen
             FROM patterns WHERE category = ?1",
        )?;

        let rows = stmt.query_map(params![category], |row| {
            Ok(Pattern {
                id: row.get(0)?,
                category: row.get(1)?,
                fingerprint: row.get(2)?,
                score: row.get(3)?,
                first_seen: row.get::<_, String>(4)?.parse().unwrap(),
                last_seen: row.get::<_, String>(5)?.parse().unwrap(),
            })
        })?;

        Ok(rows.map(|r| r.unwrap()).collect())
    }

    /// Update last_seen & score for pattern ID
    pub fn update_score_and_seen(
        &self,
        id: i64,
        new_score: f64,
    ) -> Result<(), PatternCacheError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE patterns
             SET score = ?1, last_seen = ?2
             WHERE id = ?3",
            params![new_score, Utc::now().to_rfc3339(), id],
        )?;

        Ok(())
    }
}
