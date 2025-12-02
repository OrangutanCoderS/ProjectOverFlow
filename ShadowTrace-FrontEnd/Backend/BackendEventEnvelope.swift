//
//  BackendEventEnvelope.swift
//  ShadowTrace
//

import Foundation

/// Unified envelope used throughout ShadowTrace.
///
/// ShadowTrace v1 backend shape:
/// {
///   "type": "cpu_stats",
///   "timestamp": 1732440034,
///   "payload": { ... }
/// }
///
/// OverFlow / system crate (Rust AnyEvent streamer):
/// {
///   "event_type": "cpu",
///   "ts_ms": 1764509195961,
///   "data": { ... }
/// }
///
/// This struct normalizes both into a canonical form:
///   - type: logical event kind ("cpu_stats", "memory_stats", "cpu", "system_stats", ...)
///   - timestamp: seconds since epoch
///   - payload: raw JSONValue object
struct BackendEventEnvelope: Decodable {
    let type: String
    let timestamp: TimeInterval
    let payload: JSONValue

    private enum CodingKeys: String, CodingKey {
        // ShadowTrace keys
        case type
        case timestamp
        case payload

        // OverFlow / system crate keys
        case eventType = "event_type"
        case data
        case tsMs = "ts_ms"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // ----------------------------
        // PATH 1 — Native ShadowTrace
        // ----------------------------
        if let t = try container.decodeIfPresent(String.self, forKey: .type) {
            self.type = t
            self.payload = try container.decode(JSONValue.self, forKey: .payload)

            // If timestamp missing → default to now
            self.timestamp =
                (try container.decodeIfPresent(TimeInterval.self, forKey: .timestamp))
                ?? Date().timeIntervalSince1970

            return
        }

        // --------------------------------------------
        // PATH 2 — OverFlow / system crate (Rust JSON)
        // --------------------------------------------
        let rawType = try container.decode(String.self, forKey: .eventType)
        self.type = rawType                           // "cpu", "system_stats", "gpu", "battery"
        self.payload = try container.decode(JSONValue.self, forKey: .data)

        // Convert ts_ms → seconds
        if let ms = try container.decodeIfPresent(Double.self, forKey: .tsMs) {
            self.timestamp = ms / 1000.0
        } else {
            self.timestamp = Date().timeIntervalSince1970
        }
    }
}
