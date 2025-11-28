use trace_timeline::{
    EventKind, TimelineCursor, TimelineEvent, TimelineSource, TraceTimelineConfig,
    TraceTimelineStore,
};

fn make_event(ts: i64) -> TimelineEvent {
    TimelineEvent::new(ts, EventKind::MetricSample, TimelineSource::SystemMetrics)
}

#[test]
fn events_are_stored_sorted() {
    let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());

    store.insert(make_event(30)).unwrap();
    store.insert(make_event(10)).unwrap();
    store.insert(make_event(20)).unwrap();

    let ts: Vec<_> = store.events().iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts, vec![10, 20, 30]);
}

#[test]
fn window_by_range_behaves() {
    let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());

    for ts in [10, 20, 30, 40, 50] {
        store.insert(make_event(ts)).unwrap();
    }

    let w = store.window_by_range(15, 45);
    let ts: Vec<_> = w.iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts, vec![20, 30, 40]);

    let empty = store.window_by_range(100, 200);
    assert!(empty.is_empty());
}

#[test]
fn last_n_works() {
    let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());
    for ts in [10, 20, 30, 40, 50] {
        store.insert(make_event(ts)).unwrap();
    }

    let last_two = store.last_n(2);
    let ts: Vec<_> = last_two.iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts, vec![40, 50]);

    let more_than_len = store.last_n(10);
    assert_eq!(more_than_len.len(), 5);
}

#[test]
fn around_center_respects_radius() {
    let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());
    for ts in [0, 10, 20, 30, 40, 50] {
        store.insert(make_event(ts)).unwrap();
    }

    // radius 9 around 25 -> [16, 34] -> hits 20 and 30
    let w = store.around(25, 9);
    let ts: Vec<_> = w.iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts, vec![20, 30]);

    // big radius hits almost everything
    let w2 = store.around(25, 30);
    assert_eq!(w2.len(), store.len());
}

#[test]
fn capacity_without_eviction_fails() {
    let cfg = TraceTimelineConfig::default()
        .with_max_events(3)
        .with_evict_on_overflow(false);

    let mut store = TraceTimelineStore::new(cfg);

    store.insert(make_event(10)).unwrap();
    store.insert(make_event(20)).unwrap();
    store.insert(make_event(30)).unwrap();

    let res = store.insert(make_event(40));
    assert!(res.is_err());
}

#[test]
fn capacity_with_eviction_drops_oldest() {
    let cfg = TraceTimelineConfig::default()
        .with_max_events(3)
        .with_evict_on_overflow(true);

    let mut store = TraceTimelineStore::new(cfg);

    store.insert(make_event(10)).unwrap();
    store.insert(make_event(20)).unwrap();
    store.insert(make_event(30)).unwrap();
    store.insert(make_event(40)).unwrap();

    let ts: Vec<_> = store.events().iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts, vec![20, 30, 40]);
}

#[test]
fn cursor_slides_over_store() {
    let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());
    for ts in [10, 20, 30, 40, 50] {
        store.insert(make_event(ts)).unwrap();
    }

    let mut cursor = TimelineCursor::new(&store);

    let w1 = cursor.next_by_count(2).unwrap();
    let ts1: Vec<_> = w1.events().iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts1, vec![10, 20]);

    let w2 = cursor.next_by_count(2).unwrap();
    let ts2: Vec<_> = w2.events().iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts2, vec![30, 40]);

    let w3 = cursor.next_by_count(2).unwrap();
    let ts3: Vec<_> = w3.events().iter().map(|e| e.ts_nanos).collect();
    assert_eq!(ts3, vec![50]);

    assert!(cursor.next_by_count(2).is_none());
    assert!(cursor.is_finished());

    cursor.reset();
    assert!(!cursor.is_finished());
    assert_eq!(cursor.position(), 0);
}