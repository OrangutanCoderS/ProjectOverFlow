//
//  BatteryHistoryPoint.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  BatteryStatsAdapter.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation
import Combine

/// One battery sample in time.
struct BatteryHistoryPoint: Identifiable, Codable {
    let id: UUID
    let timestamp: TimeInterval

    let levelPercent: Double          // 0–100
    let isCharging: Bool
    let onACPower: Bool

    let healthPercent: Double?
    let cycleCount: Int?
    let temperatureC: Double?
    let powerWatts: Double?

    init(
        id: UUID = UUID(),
        timestamp: TimeInterval,
        levelPercent: Double,
        isCharging: Bool,
        onACPower: Bool,
        healthPercent: Double? = nil,
        cycleCount: Int? = nil,
        temperatureC: Double? = nil,
        powerWatts: Double? = nil
    ) {
        self.id = id
        self.timestamp = timestamp
        self.levelPercent = levelPercent
        self.isCharging = isCharging
        self.onACPower = onACPower
        self.healthPercent = healthPercent
        self.cycleCount = cycleCount
        self.temperatureC = temperatureC
        self.powerWatts = powerWatts
    }
}

/// Bridges backend battery stats into:
/// - latest snapshot
/// - rolling history for the sparkline.
@MainActor
final class BatteryStatsAdapter: ObservableObject {

    static let shared = BatteryStatsAdapter()

    /// Newest sample at the end.
    @Published private(set) var history: [BatteryHistoryPoint] = []

    /// Hard cap to avoid unbounded growth.
    private let maxHistoryPoints: Int = 300

    private init() {}

    // MARK: - Public API

    /// Generic update API. You will call this from backend glue.
    func update(
        levelPercent: Double,
        isCharging: Bool,
        onACPower: Bool,
        healthPercent: Double? = nil,
        cycleCount: Int? = nil,
        temperatureC: Double? = nil,
        powerWatts: Double? = nil
    ) {
        let clamped = max(0, min(levelPercent, 100))
        let now = Date().timeIntervalSince1970

        let point = BatteryHistoryPoint(
            timestamp: now,
            levelPercent: clamped,
            isCharging: isCharging,
            onACPower: onACPower,
            healthPercent: healthPercent,
            cycleCount: cycleCount,
            temperatureC: temperatureC,
            powerWatts: powerWatts
        )

        history.append(point)
        trimIfNeeded()
    }

    /// Convenience hook if later you want to feed this from `SensorUpdateEvent`.
    /// Safe no-op unless you call it yourself.
    func update(from sensor: SensorUpdateEvent) {
        // Rust AnyEvent Battery does NOT come through SensorUpdateEvent.
        // But if someone plugs in a future “battery” sensor:
        guard sensor.kind.lowercased().contains("battery") else { return }

        let level = sensor.value ?? 0

        update(
            levelPercent: level,
            isCharging: false,
            onACPower: false,
            healthPercent: nil,
            cycleCount: nil,
            temperatureC: nil,
            powerWatts: nil
        )
    }

    /// Latest sample (used by BatteryView).
    var latest: BatteryHistoryPoint? {
        history.last
    }

    /// Used only by previews / tests to seed mock data.
    func reset(with history: [BatteryHistoryPoint]) {
        self.history = history
        trimIfNeeded()
    }

    // MARK: - Internal

    private func trimIfNeeded() {
        let overflow = history.count - maxHistoryPoints
        if overflow > 0 {
            history.removeFirst(overflow)
        }
    }
}
