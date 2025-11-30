//
//  EventDecoder.swift
//  ShadowTrace
//

import Foundation

/// Converts a normalized `BackendEventEnvelope` into a strongly-typed `BackendEvent`.
/// Handles *both* old ShadowTrace event types and the new Rust AnyEvent format.
///
/// Rust AnyEvent looks like:
/// {
///   "event_type": "cpu",
///   "ts_ms": 1764509195961,
///   "data": { ... }
/// }
///
/// ShadowTrace looks like:
/// {
///   "type": "cpu_stats",
///   "timestamp": 1732440034,
///   "payload": { ... }
/// }
///
final class EventDecoder {

    static let shared = EventDecoder()

    private let decoder: JSONDecoder
    private let encoder: JSONEncoder

    init(
        decoder: JSONDecoder = JSONDecoder(),
        encoder: JSONEncoder = JSONEncoder()
    ) {
        self.decoder = decoder
        self.encoder = encoder

        self.decoder.keyDecodingStrategy = .convertFromSnakeCase
    }

    // MARK: - Public

    func decodeGenericEnvelope(
        envelope: BackendEventEnvelope
    ) throws -> BackendEvent {

        let ts = envelope.timestamp

        let payloadData: Data
        do {
            payloadData = try encoder.encode(envelope.payload)
        } catch {
            throw EventDecoderError.payloadDecodeFailed(
                type: envelope.type,
                underlying: error
            )
        }

        return try decode(
            rawType: envelope.type,
            timestamp: ts,
            payloadData: payloadData
        )
    }

    // MARK: - Core

    private func decode(
        rawType: String,
        timestamp: TimeInterval,
        payloadData: Data
    ) throws -> BackendEvent {

        switch rawType {

        // -----------------------------
        // CPU from Rust: "cpu"
        // CPU from old ST: "cpu_stats"
        // -----------------------------
        case "cpu", "cpu_stats":
            let p = try decodeSafe(CpuStatsEvent.self, rawType, payloadData)
            return .cpuStats(p, timestamp: timestamp)

        // -------------------------------------
        // Memory/system stats from Rust: "system_stats"
        // Old: "memory_stats"
        // -------------------------------------
        case "system_stats", "memory_stats":
            let p = try decodeSafe(MemoryStatsEvent.self, rawType, payloadData)
            return .memoryStats(p, timestamp: timestamp)

        // --------------------
        // GPU from Rust: "gpu"
        // --------------------
        case "gpu":
            let p = try decodeSafe(GpuStatsEvent.self, rawType, payloadData)
            return .gpuStats(p, timestamp: timestamp)

        // ------------------------
        // Battery from Rust: "battery"
        // ------------------------
        case "battery":
            let p = try decodeSafe(BatteryStatsEvent.self, rawType, payloadData)
            return .batteryStats(p, timestamp: timestamp)

        // ------------------------
        // File events from Rust: file event plugin
        // ------------------------
        case "file_event":
            let p = try decodeSafe(FileEvent.self, rawType, payloadData)
            return .fileEvent(p, timestamp: timestamp)

        // ST old types we keep for replay mode compatibility:
        case "process_snapshot":
            let p = try decodeSafe(ProcessSnapshotEvent.self, rawType, payloadData)
            return .processSnapshot(p, timestamp: timestamp)

        case "plugin_trigger":
            let p = try decodeSafe(PluginTriggerEvent.self, rawType, payloadData)
            return .pluginTrigger(p, timestamp: timestamp)

        case "network_stats":
            let p = try decodeSafe(NetworkStatsEvent.self, rawType, payloadData)
            return .networkStats(p, timestamp: timestamp)

        case "policy_update":
            let p = try decodeSafe(PolicyUpdateEvent.self, rawType, payloadData)
            return .policyUpdate(p, timestamp: timestamp)

        case "audit_log_entry":
            let p = try decodeSafe(AuditLogEntry.self, rawType, payloadData)
            return .auditLogEntry(p, timestamp: timestamp)

        case "timeline_chunk":
            let p = try decodeSafe(TimelineChunkEvent.self, rawType, payloadData)
            return .timelineChunk(p, timestamp: timestamp)

        case "replay_progress":
            let p = try decodeSafe(ReplayProgressEvent.self, rawType, payloadData)
            return .replayProgress(p, timestamp: timestamp)

        case "version_info":
            let p = try decodeSafe(VersionInfoEvent.self, rawType, payloadData)
            return .versionInfo(p, timestamp: timestamp)

        case "error":
            let p = try decodeSafe(ErrorEvent.self, rawType, payloadData)
            return .errorEvent(p, timestamp: timestamp)

        // -----------------------------
        // Unknown = safe fallback
        // -----------------------------
        default:
            return .unknown(
                UnknownEvent(rawType: rawType),
                timestamp: timestamp
            )
        }
    }

    // MARK: - Helpers

    private func decodeSafe<T: Decodable>(
        _ type: T.Type,
        _ rawType: String,
        _ data: Data
    ) throws -> T {
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw EventDecoderError.payloadDecodeFailed(
                type: rawType,
                underlying: error
            )
        }
    }
}
