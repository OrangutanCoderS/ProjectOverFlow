//
//  ProcessCpuHistoryStore.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessStateManager+CPUHistory.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Internal mutable store for per-PID CPU histories.
/// This is intentionally decoupled from ProcessStateManager's core implementation
/// so we don't touch its main file or break existing logic.
final class ProcessCpuHistoryStore {

    static let shared = ProcessCpuHistoryStore()

    /// In-memory store keyed by PID.
    private var histories: [Int: MutableHistory] = [:]

    /// Hard cap on samples per PID to avoid unbounded memory growth.
    private let maxSamplesPerPid = 300

    private struct MutableHistory {
        var pid: Int
        var name: String?
        var samples: [CpuProcessSample]
    }

    private let lock = NSLock()

    private init() {}

    func record(pid: Int, name: String?, cpuPercent: Double, timestamp: TimeInterval) {
        lock.lock()
        defer { lock.unlock() }

        var history = histories[pid] ?? MutableHistory(pid: pid, name: name, samples: [])
        history.name = name ?? history.name

        let sample = CpuProcessSample(timestamp: timestamp, cpuPercent: cpuPercent)
        history.samples.append(sample)

        // Trim if needed
        let overflow = history.samples.count - maxSamplesPerPid
        if overflow > 0 {
            history.samples.removeFirst(overflow)
        }

        histories[pid] = history
    }

    func history(for pid: Int) -> CpuProcessHistory? {
        lock.lock()
        defer { lock.unlock() }

        guard let h = histories[pid] else { return nil }
        return CpuProcessHistory(pid: h.pid, name: h.name, samples: h.samples)
    }

    func lastCpu(for pid: Int) -> Double? {
        history(for: pid)?.lastCpuPercent
    }

    func peakCpu(for pid: Int) -> Double? {
        history(for: pid)?.peakCpuPercent
    }

    func averageCpu(for pid: Int) -> Double? {
        history(for: pid)?.averageCpuPercent
    }

    func lastTimestamp(for pid: Int) -> TimeInterval? {
        history(for: pid)?.lastTimestamp
    }

    func clearAll() {
        lock.lock()
        histories.removeAll()
        lock.unlock()
    }
}

// MARK: - Public extension surface for ProcessStateManager

@MainActor
extension ProcessStateManager {

    /// Read-only view used by CpuProcessView.
    func cpuHistory(for pid: Int) -> CpuProcessHistory? {
        ProcessCpuHistoryStore.shared.history(for: pid)
    }

    func lastCpu(for pid: Int) -> Double? {
        ProcessCpuHistoryStore.shared.lastCpu(for: pid)
    }

    func peakCpu(for pid: Int) -> Double? {
        ProcessCpuHistoryStore.shared.peakCpu(for: pid)
    }

    func averageCpu(for pid: Int) -> Double? {
        ProcessCpuHistoryStore.shared.averageCpu(for: pid)
    }

    func lastCpuTimestamp(for pid: Int) -> TimeInterval? {
        ProcessCpuHistoryStore.shared.lastTimestamp(for: pid)
    }

    /// Optional hook you can call from wherever you handle process snapshots:
    ///
    ///     // Example inside ProcessStateManager when a new snapshot arrives:
    ///     for proc in snapshot.processes {
    ///         recordCpuSample(from: proc, timestamp: ts)
    ///     }
    ///
    func recordCpuSample(from entry: ProcessInfoEventEntry, timestamp: TimeInterval) {
        ProcessCpuHistoryStore.shared.record(
            pid: entry.pid,
            name: entry.name,
            cpuPercent: entry.cpu,
            timestamp: timestamp
        )
    }

    /// If you ever reset processes (e.g. replay start), you can clear the history too.
    func clearCpuHistories() {
        ProcessCpuHistoryStore.shared.clearAll()
    }
}