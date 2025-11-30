//
//  DashboardCpuSnapshot.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  DashboardCpuSnapshot.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

/// Immutable snapshot of CPU state used by the dashboard CPU card.
struct DashboardCpuSnapshot {
    /// Total CPU usage in percent (0–100).
    let totalUsage: Double

    /// Per-core usage in percent, same ordering as backend `perCore`.
    let perCoreUsage: [Double]

    /// Rolling history of total usage, used for the sparkline.
    /// Oldest value first, newest last.
    let history: [Double]

    /// How many processes are currently tracked.
    let processCount: Int

    /// Timestamp of the most recent stats included in this snapshot.
    let lastUpdated: Date

    /// Convenience: placeholder while backend is not yet streaming.
    static let placeholder = DashboardCpuSnapshot(
        totalUsage: 0,
        perCoreUsage: [],
        history: [],
        processCount: 0,
        lastUpdated: Date.distantPast
    )
}