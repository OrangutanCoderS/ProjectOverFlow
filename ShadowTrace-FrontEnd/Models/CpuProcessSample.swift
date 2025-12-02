//
//  CpuProcessSample.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessHistory.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// One CPU sample for a given process at a moment in time.
struct CpuProcessSample: Codable, Identifiable {
    let id: UUID
    let timestamp: TimeInterval
    let cpuPercent: Double

    init(id: UUID = UUID(), timestamp: TimeInterval, cpuPercent: Double) {
        self.id = id
        self.timestamp = timestamp
        self.cpuPercent = cpuPercent
    }
}

/// Rolling CPU history for a single PID.
/// This is read-only from the UI side; mutation is done via the history store.
struct CpuProcessHistory: Codable {
    let pid: Int
    let name: String?

    /// Samples in chronological order (oldest first).
    let samples: [CpuProcessSample]

    /// Last sample in the history.
    var last: CpuProcessSample? {
        samples.last
    }

    /// Last CPU percent (or 0 if no samples).
    var lastCpuPercent: Double {
        last?.cpuPercent ?? 0
    }

    /// Peak CPU percent across samples.
    var peakCpuPercent: Double {
        samples.map(\.cpuPercent).max() ?? 0
    }

    /// Average CPU percent across samples.
    var averageCpuPercent: Double {
        guard !samples.isEmpty else { return 0 }
        let total = samples.reduce(0) { $0 + $1.cpuPercent }
        return total / Double(samples.count)
    }

    /// Timestamp of last sample, or nil if history is empty.
    var lastTimestamp: TimeInterval? {
        last?.timestamp
    }
}