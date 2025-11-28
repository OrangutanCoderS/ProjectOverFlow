use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::db::PatternStore;
use crate::error::TelemetryPatternError;
use crate::model::{PatternStats, TelemetryEvent};

/// Core pattern miner.
///
/// - Groups events by (session_id, process_id, origin_plugin)
/// - Sorts by timestamp
/// - Uses sliding windows over plugin names to generate chains
/// - Aggregates into PatternStats and flushes into PatternStore
pub struct PatternMiner {
    store: PatternStore,
    /// Maximum chain length (window size).
    max_chain_len: usize,
    /// Minimum occurrences before we persist a pattern.
    min_support: u64,
}

impl PatternMiner {
    /// Construct with default parameters (max_chain_len=4, min_support=3).
    pub fn new(store: PatternStore) -> Self {
        Self {
            store,
            max_chain_len: 4,
            min_support: 3,
        }
    }

    /// Construct with explicit parameters.
    pub fn with_params(
        store: PatternStore,
        max_chain_len: usize,
        min_support: u64,
    ) -> Self {
        Self {
            store,
            max_chain_len: max_chain_len.max(1),
            min_support: min_support.max(1),
        }
    }

    /// Access the underlying store.
    pub fn store(&self) -> &PatternStore {
        &self.store
    }

    /// Mine patterns from a batch of telemetry events.
    ///
    /// This is designed to be called on:
    /// - a time window (e.g., last 30 minutes), or
    /// - a replayed session from memory_replay_engine.
    pub fn mine_batch(
        &self,
        events: &[TelemetryEvent],
    ) -> Result<(), TelemetryPatternError> {
        let mut buckets: HashMap<BucketKey, Vec<&TelemetryEvent>> = HashMap::new();

        // 1. Group.
        for ev in events {
            let key = BucketKey::from_event(ev);
            buckets.entry(key).or_default().push(ev);
        }

        // 2. For each bucket, sort by time and generate patterns.
        let mut accum: HashMap<String, PatternAccum> = HashMap::new();

        for (_key, mut evs) in buckets {
            evs.sort_by_key(|e| e.timestamp);
            self.process_sequence(&evs, &mut accum)?;
        }

        // 3. Convert accumulators to PatternStats and upsert into store.
        for (fp, acc) in accum {
            if acc.count < self.min_support {
                continue;
            }
            let stats = acc.into_stats(fp);
            debug!(
                "PatternMiner: persisting pattern {:?} count={}",
                stats.sample_chain, stats.count
            );
            self.store.upsert_pattern(&stats)?;
        }

        Ok(())
    }

    fn process_sequence(
        &self,
        events: &[&TelemetryEvent],
        accum: &mut HashMap<String, PatternAccum>,
    ) -> Result<(), TelemetryPatternError> {
        if events.is_empty() {
            return Ok(());
        }

        // Extract plugin names and timestamps once.
        let plugins: Vec<&str> = events.iter().map(|e| e.plugin.as_str()).collect();
        let timestamps: Vec<DateTime<Utc>> =
            events.iter().map(|e| e.timestamp).collect();

        for start in 0..plugins.len() {
            for len in 2..=self.max_chain_len {
                let end = start + len;
                if end > plugins.len() {
                    break;
                }

                let slice = &plugins[start..end];
                let chain: Vec<String> = slice.iter().map(|p| (*p).to_string()).collect();
                let fingerprint = fingerprint_chain(&chain);

                let first_ts = timestamps[start];
                let last_ts = timestamps[end - 1];

                // Basic entropy / failure aggregation for this chain occurrence.
                let mut entropy_sum = 0.0;
                let mut entropy_count = 0u64;
                let mut failure = false;

                for ev in &events[start..end] {
                    if let Some(e) = ev.entropy {
                        entropy_sum += e;
                        entropy_count += 1;
                    }
                    if ev.failed {
                        failure = true;
                    }
                }

                let entry = accum.entry(fingerprint).or_insert_with(|| PatternAccum {
                    first_seen: first_ts,
                    last_seen: last_ts,
                    count: 0,
                    entropy_sum: 0.0,
                    entropy_count: 0,
                    failures: 0,
                    sample_chain: chain.clone(),
                });

                entry.count += 1;
                if first_ts < entry.first_seen {
                    entry.first_seen = first_ts;
                }
                if last_ts > entry.last_seen {
                    entry.last_seen = last_ts;
                }

                if entropy_count > 0 {
                    entry.entropy_sum += entropy_sum;
                    entry.entropy_count += entropy_count;
                }

                if failure {
                    entry.failures += 1;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct BucketKey {
    session_id: String,
    process_id: i64,
    origin_plugin: String,
}

impl BucketKey {
    fn from_event(ev: &TelemetryEvent) -> Self {
        Self {
            session_id: ev.session_id.clone().unwrap_or_else(|| "default".to_string()),
            process_id: ev.process_id.unwrap_or(-1),
            origin_plugin: ev.origin_plugin.clone().unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

struct PatternAccum {
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    count: u64,
    entropy_sum: f64,
    entropy_count: u64,
    failures: u64,
    sample_chain: Vec<String>,
}

impl PatternAccum {
    fn into_stats(self, fingerprint: String) -> PatternStats {
        let avg_entropy = if self.entropy_count > 0 {
            Some(self.entropy_sum / (self.entropy_count as f64))
        } else {
            None
        };

        let failure_rate = if self.count > 0 {
            Some(self.failures as f64 / self.count as f64)
        } else {
            None
        };

        PatternStats {
            fingerprint,
            length: self.sample_chain.len() as u32,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            count: self.count,
            avg_entropy,
            failure_rate,
            sample_chain: self.sample_chain,
        }
    }
}

fn fingerprint_chain(chain: &[String]) -> String {
    let mut hasher = Sha256::new();
    for name in chain {
        hasher.update(name.as_bytes());
        hasher.update(b"|");
    }
    let digest = hasher.finalize();
    // Manual hex encoding to avoid extra deps.
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", b);
    }
    out
}