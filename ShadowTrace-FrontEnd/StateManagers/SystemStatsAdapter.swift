//
//  SystemStatsAdapter.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import Combine

// MARK: - History Point Models

/// One point in time for CPU usage, suitable for graphs & replay.
struct CpuHistoryPoint: Codable, Identifiable {
    let id: UUID
    let timestamp: TimeInterval

    let userPercent: Double
    let systemPercent: Double
    let idlePercent: Double
    let perCore: [Double]

    init(
        id: UUID = UUID(),
        timestamp: TimeInterval,
        userPercent: Double,
        systemPercent: Double,
        idlePercent: Double,
        perCore: [Double]
    ) {
        self.id = id
        self.timestamp = timestamp
        self.userPercent = userPercent
        self.systemPercent = systemPercent
        self.idlePercent = idlePercent
        self.perCore = perCore
    }

    init(event: CpuStatsEvent, timestamp: TimeInterval) {
        self.init(
            timestamp: timestamp,
            userPercent: event.userPercent,
            systemPercent: event.systemPercent,
            idlePercent: event.idlePercent,
            perCore: event.perCore
        )
    }
}

/// One point in time for Memory usage.
struct MemoryHistoryPoint: Codable, Identifiable {
    let id: UUID
    let timestamp: TimeInterval

    let totalMb: Double
    let usedMb: Double
    let freeMb: Double
    let wiredMb: Double?
    let cachedMb: Double?

    init(
        id: UUID = UUID(),
        timestamp: TimeInterval,
        totalMb: Double,
        usedMb: Double,
        freeMb: Double,
        wiredMb: Double?,
        cachedMb: Double?
    ) {
        self.id = id
        self.timestamp = timestamp
        self.totalMb = totalMb
        self.usedMb = usedMb
        self.freeMb = freeMb
        self.wiredMb = wiredMb
        self.cachedMb = cachedMb
    }

    init(event: MemoryStatsEvent, timestamp: TimeInterval) {
        self.init(
            timestamp: timestamp,
            totalMb: event.totalMb,
            usedMb: event.usedMb,
            freeMb: event.freeMb,
            wiredMb: event.wiredMb,
            cachedMb: event.cachedMb
        )
    }
}

// MARK: - Disk Snapshot Helper (internal to this file)

/// Responsible ONLY for persisting and loading CPU/Memory history
/// to small JSON files under Application Support.
/// This keeps SystemStatsAdapter focused on in-memory state.
fileprivate struct SystemStatsDiskStore {

    static let shared = SystemStatsDiskStore()

    private let fileManager = FileManager.default

    private var baseDirectory: URL {
        let appSupport = fileManager.urls(for: .applicationSupportDirectory,
                                          in: .userDomainMask).first!
        let dir = appSupport.appendingPathComponent("ShadowTrace", isDirectory: true)

        if !fileManager.fileExists(atPath: dir.path) {
            try? fileManager.createDirectory(at: dir,
                                             withIntermediateDirectories: true,
                                             attributes: nil)
        }
        return dir
    }

    private var cpuHistoryURL: URL {
        baseDirectory.appendingPathComponent("cpu_history.json", isDirectory: false)
    }

    private var memoryHistoryURL: URL {
        baseDirectory.appendingPathComponent("memory_history.json", isDirectory: false)
    }

    // MARK: Load

    func loadCpuHistory() -> [CpuHistoryPoint] {
        guard fileManager.fileExists(atPath: cpuHistoryURL.path) else {
            return []
        }
        do {
            let data = try Data(contentsOf: cpuHistoryURL)
            let decoder = JSONDecoder()
            return try decoder.decode([CpuHistoryPoint].self, from: data)
        } catch {
            print("[SystemStatsDiskStore] Failed to load CPU history: \(error)")
            return []
        }
    }

    func loadMemoryHistory() -> [MemoryHistoryPoint] {
        guard fileManager.fileExists(atPath: memoryHistoryURL.path) else {
            return []
        }
        do {
            let data = try Data(contentsOf: memoryHistoryURL)
            let decoder = JSONDecoder()
            return try decoder.decode([MemoryHistoryPoint].self, from: data)
        } catch {
            print("[SystemStatsDiskStore] Failed to load memory history: \(error)")
            return []
        }
    }

    // MARK: Save (async, non-blocking)

    func saveCpuHistory(_ points: [CpuHistoryPoint]) {
        let snapshot = points
        let url = cpuHistoryURL

        Task.detached {
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.sortedKeys]

            do {
                let data = try encoder.encode(snapshot)
                try data.write(to: url, options: [.atomic])
            } catch {
                print("[SystemStatsDiskStore] Failed to save CPU history: \(error)")
            }
        }
    }

    func saveMemoryHistory(_ points: [MemoryHistoryPoint]) {
        let snapshot = points
        let url = memoryHistoryURL

        Task.detached {
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.sortedKeys]

            do {
                let data = try encoder.encode(snapshot)
                try data.write(to: url, options: [.atomic])
            } catch {
                print("[SystemStatsDiskStore] Failed to save memory history: \(error)")
            }
        }
    }
}

// MARK: - SystemStatsAdapter

/// Bridges backend `CpuStatsEvent` / `MemoryStatsEvent` into:
/// - live "latest" values for cards
/// - rolling history for graphs
/// - lightweight snapshot persistence to disk
@MainActor
final class SystemStatsAdapter: ObservableObject {

    static let shared = SystemStatsAdapter()

    // MARK: Published state used by SwiftUI

    @Published private(set) var latestCpu: CpuStatsEvent?
    @Published private(set) var latestMemory: MemoryStatsEvent?

    @Published private(set) var cpuHistory: [CpuHistoryPoint]
    @Published private(set) var memoryHistory: [MemoryHistoryPoint]

    /// Hard cap for in-memory history to avoid unbounded growth.
    /// Example: 300 points ≈ 5 minutes at 1 Hz.
    private let maxHistoryPoints: Int = 300

    private let diskStore: SystemStatsDiskStore

    // MARK: Init

    @MainActor
    init() {
        self.diskStore = .shared
        // Warm start from disk snapshots (if any).
        self.cpuHistory = self.diskStore.loadCpuHistory()
        self.memoryHistory = self.diskStore.loadMemoryHistory()
    }

    fileprivate init(diskStore: SystemStatsDiskStore) {
        self.diskStore = diskStore
        // Warm start from disk snapshots (if any).
        self.cpuHistory = diskStore.loadCpuHistory()
        self.memoryHistory = diskStore.loadMemoryHistory()
    }

    // MARK: Update from backend events

    /// Called by `BackendStateManager` when a new CpuStatsEvent arrives.
    func updateCpu(_ event: CpuStatsEvent) {
        latestCpu = event

        let now = Date().timeIntervalSince1970
        let point = CpuHistoryPoint(event: event, timestamp: now)

        cpuHistory.append(point)
        trimIfNeeded(&cpuHistory)

        diskStore.saveCpuHistory(cpuHistory)
    }

    /// Called by `BackendStateManager` when a new MemoryStatsEvent arrives.
    func updateMemory(_ event: MemoryStatsEvent) {
        latestMemory = event

        let now = Date().timeIntervalSince1970
        let point = MemoryHistoryPoint(event: event, timestamp: now)

        memoryHistory.append(point)
        trimIfNeeded(&memoryHistory)

        diskStore.saveMemoryHistory(memoryHistory)
    }

    // MARK: Helper

    /// Keep the history arrays within `maxHistoryPoints`.
    private func trimIfNeeded<T>(_ array: inout [T]) {
        let overflow = array.count - maxHistoryPoints
        if overflow > 0 {
            array.removeFirst(overflow)
        }
    }
}

