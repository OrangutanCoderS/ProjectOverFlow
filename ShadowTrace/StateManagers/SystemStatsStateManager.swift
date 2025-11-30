//
//  SystemStatsStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  SystemStatsStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import Combine
/// Central, UI-friendly store for system-wide stats used by the dashboard.
/// At the moment it only exposes CPU for `DashboardCpuCard`.
@MainActor
final class SystemStatsStateManager: ObservableObject {

    static let shared = SystemStatsStateManager()

    /// Current CPU snapshot for the dashboard.
    @Published private(set) var cpuSnapshot: DashboardCpuSnapshot = .placeholder

    /// Internal history buffer of total CPU usage (0–100).
    private var totalUsageHistory: [Double] = []

    /// Maximum number of samples kept for the sparkline.
    private let maxHistorySamples = 60

    private init() {}

    /// Ingest a new CPU stats event from the adapter.
    func ingest(cpu stats: CpuStatsEvent, timestamp: TimeInterval) {
        // Convert backend fields into a single "total usage" figure.
        let used = clamp(100.0 - stats.idlePercent, min: 0.0, max: 100.0)

        totalUsageHistory.append(used)
        if totalUsageHistory.count > maxHistorySamples {
            totalUsageHistory.removeFirst(totalUsageHistory.count - maxHistorySamples)
        }

        let snapshot = DashboardCpuSnapshot(
            totalUsage: used,
            perCoreUsage: stats.perCore,
            history: totalUsageHistory,
            processCount: ProcessStateManager.shared.processCount,
            lastUpdated: Date(timeIntervalSince1970: timestamp)
        )

        cpuSnapshot = snapshot
    }

    // MARK: - Utilities

    private func clamp(_ value: Double, min: Double, max: Double) -> Double {
        if value < min { return min }
        if value > max { return max }
        return value
    }
}
