//
//  DiskProcessSample.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  DiskProcessModels.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// One timestamped sample of per-process disk I/O.
struct DiskProcessSample: Codable, Identifiable {
    let id: UUID
    let timestamp: TimeInterval

    /// Instantaneous read throughput in KB/s for this process.
    let readKBps: Double

    /// Instantaneous write throughput in KB/s for this process.
    let writeKBps: Double

    /// Optional cumulative bytes read since process start, if available.
    let totalReadBytes: Int64?

    /// Optional cumulative bytes written since process start, if available.
    let totalWrittenBytes: Int64?

    init(
        id: UUID = UUID(),
        timestamp: TimeInterval,
        readKBps: Double,
        writeKBps: Double,
        totalReadBytes: Int64? = nil,
        totalWrittenBytes: Int64? = nil
    ) {
        self.id = id
        self.timestamp = timestamp
        self.readKBps = readKBps
        self.writeKBps = writeKBps
        self.totalReadBytes = totalReadBytes
        self.totalWrittenBytes = totalWrittenBytes
    }
}

/// Rolling history of disk activity for a single process (identified by PID).
struct DiskProcessHistory: Identifiable {
    let pid: Int
    let processName: String

    /// Samples in chronological order (oldest first, newest last).
    var samples: [DiskProcessSample]

    var id: Int { pid }

    /// Latest sample if available.
    var latest: DiskProcessSample? {
        samples.last
    }

    /// Convenience: last N samples for sparkline.
    func tail(_ count: Int) -> [DiskProcessSample] {
        guard count < samples.count else { return samples }
        return Array(samples.suffix(count))
    }
}