//
//  NetworkStatsAdapter.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation
import Combine

// MARK: - History point

/// A single point representing *computed* live network throughput.
struct NetworkHistoryPoint: Codable, Identifiable {
    let id: UUID
    let timestamp: TimeInterval

    let uploadKBps: Double
    let downloadKBps: Double
    let activeConnections: Int
    let droppedPackets: Int

    init(
        id: UUID = UUID(),
        timestamp: TimeInterval,
        uploadKBps: Double,
        downloadKBps: Double,
        activeConnections: Int,
        droppedPackets: Int
    ) {
        self.id = id
        self.timestamp = timestamp
        self.uploadKBps = uploadKBps
        self.downloadKBps = downloadKBps
        self.activeConnections = activeConnections
        self.droppedPackets = droppedPackets
    }
}

// MARK: - Adapter

/// Bridges backend `NetworkStatsEvent` (total bytes since boot) into:
/// - computed KB/s snapshot for the dashboard
/// - rolling history for graphs
@MainActor
final class NetworkStatsAdapter: ObservableObject {

    static let shared = NetworkStatsAdapter()

    // Latest computed snapshot (NOT raw backend event)
    @Published private(set) var latestSnapshot: NetworkSnapshot?

    // Rolling history
    @Published private(set) var history: [NetworkHistoryPoint] = []

    /// Previous raw event for computing deltas
    private var lastRaw: (event: NetworkStatsEvent, timestamp: TimeInterval)?

    /// Hard cap
    private let maxHistoryPoints: Int = 300

    private init() {}

    /// Snapshot used by UI components
    struct NetworkSnapshot {
        let uploadKBps: Double
        let downloadKBps: Double
        let activeConnections: Int
        let droppedPackets: Int
        let totalRateString: String
    }

    // MARK: - Update on backend event
    func update(_ event: NetworkStatsEvent, timestamp: TimeInterval) {

        // FIRST TIME → no delta yet
        guard let last = lastRaw else {
            lastRaw = (event, timestamp)
            latestSnapshot = NetworkSnapshot(
                uploadKBps: 0,
                downloadKBps: 0,
                activeConnections: event.activeConnections ?? 0,
                droppedPackets: event.droppedPackets ?? 0,
                totalRateString: "0 KB/s"
            )
            return
        }

        let dt = max(0.001, timestamp - last.timestamp)

        let deltaSent = Double(event.bytesSent - last.event.bytesSent)
        let deltaRecv = Double(event.bytesReceived - last.event.bytesReceived)

        let uploadKBps = max(0, deltaSent / dt / 1024)
        let downloadKBps = max(0, deltaRecv / dt / 1024)

        let total = uploadKBps + downloadKBps
        let rateStr = String(format: "%.1f KB/s", total)

        let snapshot = NetworkSnapshot(
            uploadKBps: uploadKBps,
            downloadKBps: downloadKBps,
            activeConnections: event.activeConnections ?? 0,
            droppedPackets: event.droppedPackets ?? 0,
            totalRateString: rateStr
        )

        latestSnapshot = snapshot
        lastRaw = (event, timestamp)

        // push history point
        let point = NetworkHistoryPoint(
            timestamp: timestamp,
            uploadKBps: uploadKBps,
            downloadKBps: downloadKBps,
            activeConnections: snapshot.activeConnections,
            droppedPackets: snapshot.droppedPackets
        )

        history.append(point)
        trim()
    }

    // MARK: - Helpers
    private func trim() {
        let overflow = history.count - maxHistoryPoints
        if overflow > 0 {
            history.removeFirst(overflow)
        }
    }
}
