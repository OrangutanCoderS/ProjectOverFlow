//
//  EnergyProcessHistory.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  EnergyProcessHistory.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Rolling history of energy stats for a single process (pid).
///
/// This is intentionally compact: we track the full history only for the
/// normalized energy score; other metrics keep just the latest sample.
struct EnergyProcessHistory {

    // Identity
    let pid: Int
    let processName: String?

    // Time-series for energy score (0–100).
    var energyScoreHistory: [Double]
    var timestamps: [Date]

    // Latest breakdown snapshot.
    var currentCpuContributionPercent: Double
    var currentGpuContributionPercent: Double
    var currentThermalImpact: ThermalImpact
    var currentWakeupsPerSec: Int?
    var currentMilliwatts: Double?

    // MARK: - Derived metrics

    var lastUpdated: Date? {
        timestamps.last
    }

    var currentEnergyScore: Double {
        energyScoreHistory.last ?? 0
    }

    var peakEnergyScore: Double {
        energyScoreHistory.max() ?? 0
    }

    var averageEnergyScore: Double {
        guard !energyScoreHistory.isEmpty else { return 0 }
        let sum = energyScoreHistory.reduce(0, +)
        return sum / Double(energyScoreHistory.count)
    }

    /// Whether this history should be considered "stale" for UI purposes.
    func isStale(reference now: Date = Date(), thresholdSeconds: TimeInterval = 10) -> Bool {
        guard let last = lastUpdated else { return true }
        return now.timeIntervalSince(last) > thresholdSeconds
    }

    // MARK: - Init helpers

    static func empty(pid: Int, processName: String?) -> EnergyProcessHistory {
        EnergyProcessHistory(
            pid: pid,
            processName: processName,
            energyScoreHistory: [],
            timestamps: [],
            currentCpuContributionPercent: 0,
            currentGpuContributionPercent: 0,
            currentThermalImpact: .low,
            currentWakeupsPerSec: nil,
            currentMilliwatts: nil
        )
    }

    // MARK: - Mutation helpers

    mutating func appendSample(
        timestamp: Date,
        energyScore: Double,
        cpuContribution: Double,
        gpuContribution: Double,
        thermal: ThermalImpact,
        wakeupsPerSec: Int?,
        estimatedMilliwatts: Double?
    ) {
        let clampedScore = max(0, min(energyScore, 100))

        energyScoreHistory.append(clampedScore)
        timestamps.append(timestamp)

        currentCpuContributionPercent = cpuContribution
        currentGpuContributionPercent = gpuContribution
        currentThermalImpact = thermal
        currentWakeupsPerSec = wakeupsPerSec
        currentMilliwatts = estimatedMilliwatts
    }

    mutating func trimToLast(_ maxCount: Int) {
        guard maxCount > 0 else {
            energyScoreHistory.removeAll(keepingCapacity: false)
            timestamps.removeAll(keepingCapacity: false)
            return
        }

        if energyScoreHistory.count > maxCount {
            let overflow = energyScoreHistory.count - maxCount
            energyScoreHistory.removeFirst(overflow)
        }
        if timestamps.count > maxCount {
            let overflow = timestamps.count - maxCount
            timestamps.removeFirst(overflow)
        }
    }
}