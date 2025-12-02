//
//  EnergyProcessStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Combine
import SwiftUI

/// Central store for per-process energy histories.
///
/// Views observe this through `@ObservedObject` or `@StateObject`.
/// Backend feeds this manager via EventDecoder → BackendStateManager.
@MainActor
final class EnergyProcessStateManager: ObservableObject {

    static let shared = EnergyProcessStateManager()

    /// pid → rolling energy history
    @Published private(set) var histories: [Int: EnergyProcessHistory] = [:]

    /// Maximum samples stored per process to prevent memory bloating.
    private let maxPointsPerProcess: Int = 300

    private init() {}

    // MARK: - PUBLIC API
    // --------------------------------------------------------------

    /// Called by the backend integration whenever a new
    /// `EnergyStatsEvent` arrives for a given process.
    func update(from event: EnergyStatsEvent, timestamp: TimeInterval) {
        let pid = event.pid
        let processName = event.processName

        // Fetch history or create an empty baseline
        var history = histories[pid] ?? EnergyProcessHistory.empty(
            pid: pid,
            processName: processName
        )

        // Append new sample
        history.appendSample(
            timestamp: Date(timeIntervalSince1970: timestamp),
            energyScore: event.energyScore,
            cpuContribution: event.cpuPercent,
            gpuContribution: event.gpuPercent,
            thermal: event.thermal,
            wakeupsPerSec: event.wakeupsPerSec,
            estimatedMilliwatts: event.estimatedMilliwatts
        )

        // Trim memory overhead
        history.trimToLast(maxPointsPerProcess)

        // Persist
        histories[pid] = history
    }

    /// Query history for a given process ID.
    func history(for pid: Int) -> EnergyProcessHistory? {
        histories[pid]
    }

    // MARK: - PREVIEW SUPPORT
    // --------------------------------------------------------------

    /// Allows previews (Canvas) to seed synthetic energy histories,
    /// identical to how DiskProcessView injects fake data.
    func setHistory(_ history: EnergyProcessHistory, for pid: Int) {
        histories[pid] = history
    }
}
