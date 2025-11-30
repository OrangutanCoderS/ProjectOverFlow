//
//  DiskProcessStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  DiskProcessStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation
import Combine

/// Central store for per-process disk I/O histories.
///
/// Design:
///  - Keyed by PID
///  - Each entry holds a rolling `DiskProcessHistory`
///  - No backend wiring yet; safe to keep idle until Phase-next.
@MainActor
final class DiskProcessStateManager: ObservableObject {

    static let shared = DiskProcessStateManager()

    /// All known disk histories keyed by PID.
    @Published private(set) var histories: [Int: DiskProcessHistory] = [:]

    /// Hard cap on samples per process to avoid unbounded growth.
    private let maxSamplesPerProcess: Int = 300

    private init() {}

    // MARK: - Public API

    /// Record a new sample for a given process.
    ///
    /// This will:
    ///  - append the sample to that PID's history
    ///  - create the history if it doesn't exist yet
    ///  - trim the history if it exceeds `maxSamplesPerProcess`
    func recordSample(
        for pid: Int,
        processName: String,
        readKBps: Double,
        writeKBps: Double,
        totalReadBytes: Int64? = nil,
        totalWrittenBytes: Int64? = nil,
        at timestamp: TimeInterval = Date().timeIntervalSince1970
    ) {
        let sample = DiskProcessSample(
            timestamp: timestamp,
            readKBps: readKBps,
            writeKBps: writeKBps,
            totalReadBytes: totalReadBytes,
            totalWrittenBytes: totalWrittenBytes
        )

        if var history = histories[pid] {
            history.samples.append(sample)
            trim(&history.samples)
            histories[pid] = history
        } else {
            let history = DiskProcessHistory(
                pid: pid,
                processName: processName,
                samples: [sample]
            )
            histories[pid] = history
        }
    }

    /// Return the history for a PID, if any.
    func history(for pid: Int) -> DiskProcessHistory? {
        histories[pid]
    }

    /// Clear a single process' history.
    func clear(for pid: Int) {
        histories[pid] = nil
    }

    /// Clear all histories.
    func clearAll() {
        histories.removeAll()
    }

    // MARK: - Helpers

    private func trim(_ samples: inout [DiskProcessSample]) {
        let overflow = samples.count - maxSamplesPerProcess
        if overflow > 0 {
            samples.removeFirst(overflow)
        }
    }
}