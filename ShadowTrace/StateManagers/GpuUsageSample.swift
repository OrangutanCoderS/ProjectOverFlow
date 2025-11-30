//
//  GpuUsageSample.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  GpuUsageStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation
import Combine

/// One GPU usage sample point.
struct GpuUsageSample: Identifiable {
    let id = UUID()
    let timestamp: Date

    /// 0–100 (% of GPU load)
    let usagePercent: Double

    /// In megabytes
    let vramUsedMB: Double?
    let vramTotalMB: Double?

    /// In °C if available
    let temperatureC: Double?
}

/// Central store for GPU usage history.
///
/// Phase-1:
/// - Purely in-memory
/// - Backend will later call `pushSample(...)` whenever a new GPU reading arrives.
@MainActor
final class GpuUsageStateManager: ObservableObject {

    static let shared = GpuUsageStateManager()

    /// Rolling history (newest last).
    @Published private(set) var samples: [GpuUsageSample] = []

    /// Hard cap to avoid unbounded growth.
    private let maxSamples: Int = 300

    private init() {}

    // MARK: - Public API

    func pushSample(
        usagePercent: Double,
        vramUsedMB: Double? = nil,
        vramTotalMB: Double? = nil,
        temperatureC: Double? = nil,
        timestamp: Date = Date()
    ) {
        let clampedUsage = max(0, min(usagePercent, 100))

        let sample = GpuUsageSample(
            timestamp: timestamp,
            usagePercent: clampedUsage,
            vramUsedMB: vramUsedMB,
            vramTotalMB: vramTotalMB,
            temperatureC: temperatureC
        )

        samples.append(sample)
        trimIfNeeded()
    }

    /// Latest GPU sample (for main card header).
    var latest: GpuUsageSample? {
        samples.last
    }

    /// Convenience: last N usage percentages as a series.
    func usageSeries(limit: Int = 80) -> [Double] {
        let series = samples.map { $0.usagePercent }
        if series.count <= limit { return series }
        return Array(series.suffix(limit))
    }

    // MARK: - Debug / Preview helpers

    /// Replace samples wholesale (used in SwiftUI previews).
    func replaceAllSamples(with newSamples: [GpuUsageSample]) {
        samples = newSamples
        trimIfNeeded()
    }

    // MARK: - Internal

    private func trimIfNeeded() {
        let overflow = samples.count - maxSamples
        if overflow > 0 {
            samples.removeFirst(overflow)
        }
    }
}