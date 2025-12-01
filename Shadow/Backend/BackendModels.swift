//  BackendModels.swift
//  ShadowTrace

import Foundation

// MARK: - Event Types

enum BackendEvent {
    case cpu(CpuEvent)
    case systemStats(SystemStatsEvent)
    case gpu(GpuEvent)
    case battery(BatteryEvent)
}

// Wrapper that reads `event_type` and dispatches to correct model.
struct BackendEnvelope: Decodable {
    let event: BackendEvent

    private enum CodingKeys: String, CodingKey {
        case eventType = "event_type"
        case data
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .eventType)

        switch type {
        case "cpu":
            let data = try container.decode(CpuEvent.self, forKey: .data)
            event = .cpu(data)

        case "system_stats":
            let data = try container.decode(SystemStatsEvent.self, forKey: .data)
            event = .systemStats(data)

        case "gpu":
            let data = try container.decode(GpuEvent.self, forKey: .data)
            event = .gpu(data)

        case "battery":
            let data = try container.decode(BatteryEvent.self, forKey: .data)
            event = .battery(data)

        default:
            // Unknown event type – throw instead of silently ignoring.
            throw DecodingError.dataCorruptedError(
                forKey: .eventType,
                in: container,
                debugDescription: "Unsupported event_type: \(type)"
            )
        }
    }
}

// MARK: - Core Event Models (mirror Rust)

struct CpuEvent: Codable {
    let id: String
    let ts_ms: Int64
    let pid: Int
    let event_source: String
    let global_usage: Double
    let per_core: [Double]
}

struct SystemStatsEvent: Codable {
    let id: String
    let ts_ms: Int64
    let pid: Int
    let event_source: String
    let cpu_pct: Double
    let mem_total_mb: Double
    let mem_used_mb: Double
    let swap_total_mb: Double
    let swap_used_mb: Double
    let load1: Double
    let load5: Double
    let load15: Double
    let uptime_s: Double
    let process_count: Int
}

struct GpuEvent: Codable {
    let id: String
    let ts_ms: Int64
    let pid: Int
    let event_source: String
    let name: String
    let usage_pct: Double
    let mem_total_mb: Double
    let mem_used_mb: Double
    let temperature_c: Double
}

struct BatteryEvent: Codable {
    let id: String
    let ts_ms: Int64
    let pid: Int
    let event_source: String
    let percentage: Double
    let charging: Bool
    let cycle_count: Int?
    let temperature_c: Double?
    let health: String?
    let voltage_mv: Double?
    let time_remaining_min: Double?
}
