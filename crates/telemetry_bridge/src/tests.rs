#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::{TelemetryBridge, BridgeConfig, FileSink, TelemetryPacket};
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn file_sink_writes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("telemetry.jsonl");

        let sink = Arc::new(FileSink::new(path.clone()).unwrap());
        let bridge = TelemetryBridge::new(BridgeConfig { cache_capacity: 8 });
        bridge.add_sink(sink);

        let pkt = TelemetryPacket::new("unit", json!({"cpu": 0.7, "mem": 1234}));
        bridge.record(pkt.clone()).unwrap();

        let body = std::fs::read_to_string(path).unwrap();
        let lines: Vec<_> = body.lines().collect();
        assert_eq!(lines.len(), 1);

        let obj: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(obj["source"], "unit");
        assert_eq!(obj["metrics"]["cpu"], 0.7);
        assert_eq!(obj["metrics"]["mem"], 1234);
    }

    #[test]
    fn cache_latest_returns_n() {
        let bridge = TelemetryBridge::new(BridgeConfig { cache_capacity: 3 });
        for i in 0..5 {
            bridge.record(TelemetryPacket::new("unit", json!({"i": i}))).unwrap();
        }
        // capacity=3 keeps last 3: i = 2,3,4
        let last = bridge.latest(10);
        assert_eq!(last.len(), 3);
        assert_eq!(last[0].metrics["i"], 2);
        assert_eq!(last[2].metrics["i"], 4);
    }

    #[test]
    fn no_cache_returns_empty() {
        let bridge = TelemetryBridge::new(BridgeConfig { cache_capacity: 0 });
        bridge.record(TelemetryPacket::new("unit", json!({"ok": true}))).unwrap();
        assert!(bridge.latest(1).is_empty());
    }
}
