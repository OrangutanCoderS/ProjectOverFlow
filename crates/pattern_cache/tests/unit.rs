use chrono::Utc;
use pattern_cache::{PatternCacheEngine, model::NewPattern};

fn make_pattern(score: f64) -> NewPattern {
    NewPattern {
        category: "cpu".to_string(),
        fingerprint: "fp-1".to_string(),
        score,
        first_seen: Utc::now(),
        last_seen: Utc::now(),
    }
}

#[test]
fn insert_and_fetch() {
    let path = "test_cache.db";
    let _ = std::fs::remove_file(path);

    let cache = PatternCacheEngine::new(path).unwrap();
    let id = cache.insert(&make_pattern(0.91)).unwrap();

    let all = cache.fetch_by_category("cpu").unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, id);
}

#[test]
fn update_score() {
    let path = "test_cache2.db";
    let _ = std::fs::remove_file(path);

    let cache = PatternCacheEngine::new(path).unwrap();
    let id = cache.insert(&make_pattern(0.5)).unwrap();

    cache.update_score_and_seen(id, 0.99).unwrap();

    let out = cache.fetch_by_category("cpu").unwrap().pop().unwrap();
    assert_eq!(out.score, 0.99);
}
