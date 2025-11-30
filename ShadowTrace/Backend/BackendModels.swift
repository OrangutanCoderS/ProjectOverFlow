//
//  BackendModels.swift
//  ShadowTrace
//

import Foundation

// MARK: - Backend Event Enum

enum BackendEvent {

    // Phase I — Live system telemetry
    case cpuStats(CpuStatsEvent, timestamp: TimeInterval)
    case memoryStats(MemoryStatsEvent, timestamp: TimeInterval)
    case gpuStats(GpuStatsEvent, timestamp: TimeInterval)
    case batteryStats(BatteryStatsEvent, timestamp: TimeInterval)
    case thermalStats(ThermalStatsEvent, timestamp: TimeInterval)

    // Phase II+ — future or replay events
    case processSnapshot(ProcessSnapshotEvent, timestamp: TimeInterval)
    case fileEvent(FileEvent, timestamp: TimeInterval)
    case sensorUpdate(SensorUpdateEvent, timestamp: TimeInterval)
    case networkStats(NetworkStatsEvent, timestamp: TimeInterval)
    case pluginTrigger(PluginTriggerEvent, timestamp: TimeInterval)
    case policyUpdate(PolicyUpdateEvent, timestamp: TimeInterval)
    case auditLogEntry(AuditLogEntry, timestamp: TimeInterval)
    case timelineChunk(TimelineChunkEvent, timestamp: TimeInterval)
    case replayProgress(ReplayProgressEvent, timestamp: TimeInterval)
    case versionInfo(VersionInfoEvent, timestamp: TimeInterval)
    case errorEvent(ErrorEvent, timestamp: TimeInterval)

    case unknown(UnknownEvent, timestamp: TimeInterval)

    /// Extract timestamp from any case
    var timestamp: TimeInterval {
        switch self {
        case .cpuStats(_, let ts),
             .memoryStats(_, let ts),
             .gpuStats(_, let ts),
             .batteryStats(_, let ts),
             .thermalStats(_, let ts),
             .processSnapshot(_, let ts),
             .fileEvent(_, let ts),
             .sensorUpdate(_, let ts),
             .networkStats(_, let ts),
             .pluginTrigger(_, let ts),
             .policyUpdate(_, let ts),
             .auditLogEntry(_, let ts),
             .timelineChunk(_, let ts),
             .replayProgress(_, let ts),
             .versionInfo(_, let ts),
             .errorEvent(_, let ts),
             .unknown(_, let ts):
            return ts
        }
    }
}

// ===========================================================
// MARK: - CPU STATS
// ===========================================================

/// Maps BOTH:
/// - Rust AnyEvent::Cpu(CpuEvent)
/// - Legacy ShadowTrace CPU stats
struct CpuStatsEvent: Decodable {
    let userPercent: Double
    let systemPercent: Double
    let idlePercent: Double
    let perCore: [Double]

    private enum CodingKeys: String, CodingKey {
        case userPercent = "user_percent"
        case systemPercent = "system_percent"
        case idlePercent = "idle_percent"

        // Rust AnyEvent keys
        case globalUsage = "global_usage"
        case perCore = "per_core"
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)

        if c.contains(.userPercent) || c.contains(.systemPercent) {
            // Legacy ST format
            let user = try c.decodeIfPresent(Double.self, forKey: .userPercent) ?? 0
            let system = try c.decodeIfPresent(Double.self, forKey: .systemPercent) ?? 0
            let idle = try c.decodeIfPresent(Double.self, forKey: .idlePercent)
                ?? max(0, 100 - user - system)

            self.userPercent = user
            self.systemPercent = system
            self.idlePercent = idle

        } else {
            // Rust: global_usage + per_core
            let global = try c.decodeIfPresent(Double.self, forKey: .globalUsage) ?? 0
            self.userPercent = global
            self.systemPercent = 0
            self.idlePercent = max(0, 100 - global)
        }

        self.perCore = try c.decodeIfPresent([Double].self, forKey: .perCore) ?? []
    }
}

// ===========================================================
// MARK: - MEMORY / SYSTEM STATS
// ===========================================================

struct MemoryStatsEvent: Decodable {
    let totalMb: Double
    let usedMb: Double
    let freeMb: Double
    let wiredMb: Double?
    let cachedMb: Double?

    let swapTotalMb: Double?
    let swapUsedMb: Double?
    let load1: Double?
    let load5: Double?
    let load15: Double?
    let uptimeS: TimeInterval?
    let processCount: Int?

    private enum CodingKeys: String, CodingKey {
        // Legacy ShadowTrace
        case totalMb = "total_mb"
        case usedMb = "used_mb"
        case freeMb = "free_mb"
        case wiredMb = "wired_mb"
        case cachedMb = "cached_mb"

        // Rust
        case memTotalMb = "mem_total_mb"
        case memUsedMb = "mem_used_mb"
        case swapTotalMb = "swap_total_mb"
        case swapUsedMb = "swap_used_mb"
        case load1
        case load5
        case load15
        case uptimeS = "uptime_s"
        case processCount = "process_count"
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)

        if c.contains(.totalMb) {
            // Legacy ST shape
            let total = try c.decode(Double.self, forKey: .totalMb)
            let used = try c.decode(Double.self, forKey: .usedMb)
            let free = try c.decodeIfPresent(Double.self, forKey: .freeMb) ?? max(0, total - used)

            self.totalMb = total
            self.usedMb = used
            self.freeMb = free
            self.wiredMb = try c.decodeIfPresent(Double.self, forKey: .wiredMb)
            self.cachedMb = try c.decodeIfPresent(Double.self, forKey: .cachedMb)

        } else {
            // Rust system stats
            let total = try c.decodeIfPresent(Double.self, forKey: .memTotalMb) ?? 0
            let used = try c.decodeIfPresent(Double.self, forKey: .memUsedMb) ?? 0

            self.totalMb = total
            self.usedMb = used
            self.freeMb = max(0, total - used)
            self.wiredMb = nil
            self.cachedMb = nil
        }

        self.swapTotalMb = try c.decodeIfPresent(Double.self, forKey: .swapTotalMb)
        self.swapUsedMb = try c.decodeIfPresent(Double.self, forKey: .swapUsedMb)
        self.load1 = try c.decodeIfPresent(Double.self, forKey: .load1)
        self.load5 = try c.decodeIfPresent(Double.self, forKey: .load5)
        self.load15 = try c.decodeIfPresent(Double.self, forKey: .load15)
        self.uptimeS = try c.decodeIfPresent(TimeInterval.self, forKey: .uptimeS)
        self.processCount = try c.decodeIfPresent(Int.self, forKey: .processCount)
    }
}

// ===========================================================
// MARK: - PROCESS SNAPSHOT
// ===========================================================

struct ProcessInfoEventEntry: Decodable, Identifiable {
    let pid: Int
    let name: String
    let cpu: Double
    let memMb: Double
    let user: String

    var id: Int { pid }
}

struct ProcessSnapshotEvent: Decodable {
    let processes: [ProcessInfoEventEntry]

    init(from decoder: Decoder) throws {
        var arr = try decoder.unkeyedContainer()
        var tmp: [ProcessInfoEventEntry] = []
        while !arr.isAtEnd {
            tmp.append(try arr.decode(ProcessInfoEventEntry.self))
        }
        self.processes = tmp
    }
}

// ===========================================================
// MARK: - FILE EVENTS
// ===========================================================

struct FileEvent: Decodable {
    let path: String
    let pid: Int
    let processName: String
    let op: String
    let flagged: Bool
}

// ===========================================================
// MARK: - GENERIC SENSOR
// ===========================================================

struct SensorUpdateEvent: Decodable {
    let kind: String
    let name: String?
    let value: Double?
    let unit: String?
}

// ===========================================================
// MARK: - NETWORK
// ===========================================================

struct NetworkStatsEvent: Decodable {
    let bytesSent: Int
    let bytesReceived: Int
    let activeConnections: Int?
    let droppedPackets: Int?
}

// ===========================================================
// MARK: - FUTURE PHASE EVENTS
// ===========================================================

struct PluginTriggerEvent: Decodable {
    let pluginId: String
    let pluginName: String?
    let trigger: String
    let status: String?
    let durationMs: Int?
    let message: String?
}

struct PolicyUpdateEvent: Decodable {
    let policyId: String
    let name: String?
    let status: String?
    let changedBy: String?
    let revision: Int?
}

struct AuditLogEntry: Decodable, Identifiable {
    let id: String
    let category: String
    let message: String
    let level: String?
    let createdAt: TimeInterval?
}

struct TimelineChunkEvent: Decodable {
    let chunkId: String
    let fromTimestamp: TimeInterval
    let toTimestamp: TimeInterval
    let eventCount: Int
}

struct ReplayProgressEvent: Decodable {
    let percent: Double
    let currentTimestamp: TimeInterval?
}

// ===========================================================
// MARK: - VERSION / ERROR
// ===========================================================

struct VersionInfoEvent: Decodable {
    let version: String
    let build: String?
}

struct ErrorEvent: Decodable {
    let code: Int?
    let message: String
    let context: String?
}

// ===========================================================
// MARK: - UNKNOWN
// ===========================================================

struct UnknownEvent {
    let rawType: String
}

// ===========================================================
// MARK: - GPU / BATTERY / THERMAL
// ===========================================================

struct GpuStatsEvent: Decodable {
    let name: String
    let usagePct: Double
    let memTotalMb: Double
    let memUsedMb: Double
    let temperatureC: Double

    private enum CodingKeys: String, CodingKey {
        case name
        case usagePct = "usage_pct"
        case memTotalMb = "mem_total_mb"
        case memUsedMb = "mem_used_mb"
        case temperatureC = "temperature_c"
    }
}

struct BatteryStatsEvent: Decodable {
    let percentage: Double
    let charging: Bool
    let cycleCount: Int?
    let temperatureC: Double?
    let health: String?
    let voltageMv: Int?
    let timeRemainingMin: Int?

    private enum CodingKeys: String, CodingKey {
        case percentage
        case charging
        case cycleCount = "cycle_count"
        case temperatureC = "temperature_c"
        case health
        case voltageMv = "voltage_mv"
        case timeRemainingMin = "time_remaining_min"
    }
}

struct ThermalStatsEvent: Decodable {
    let cpuTempC: Double?
    let gpuTempC: Double?
    let skinTempC: Double?
    let packagePowerW: Double?
    let notes: String?

    private enum CodingKeys: String, CodingKey {
        case cpuTempC = "cpu_temp_c"
        case gpuTempC = "gpu_temp_c"
        case skinTempC = "skin_temp_c"
        case packagePowerW = "package_power_w"
        case notes
    }
}
