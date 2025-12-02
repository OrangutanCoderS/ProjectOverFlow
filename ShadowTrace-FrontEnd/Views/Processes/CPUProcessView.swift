//
//  CpuProcessView 2.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Per-process CPU analytics view.
/// Intended to sit inside ProcessDetailView's "CPU" tab.
struct CpuProcessView: View {

    let pid: Int
    let processName: String?

    @EnvironmentObject private var processState: ProcessStateManager

    // MARK: - Derived snapshot

    private struct Snapshot {
        let name: String?
        let pid: Int

        let lastCpu: Double
        let averageCpu: Double
        let peakCpu: Double

        let loadLevel: CpuLoadLevel

        let samples: [CpuProcessSample]
        let recentValues: [Double]

        let threads: Int?
        let priority: Int?
        let lastDelta: TimeInterval?
        let hasData: Bool
        let isStale: Bool
    }

    private func buildSnapshot() -> Snapshot {
        let history = processState.cpuHistory(for: pid)

        let samples = history?.samples ?? []
        let lastCpu = history?.lastCpuPercent ?? 0
        let avgCpu = history?.averageCpuPercent ?? 0
        let peakCpu = history?.peakCpuPercent ?? 0

        let now = Date().timeIntervalSince1970
        let lastTs = history?.lastTimestamp
        let delta = lastTs.map { now - $0 }
        let isStale = (delta ?? .infinity) > 5.0

        let loadLevel = CpuLoadLevel(percent: lastCpu)

        // Recent mini-window: last 20 samples.
        let recentValues = Array(samples.suffix(20)).map(\.cpuPercent)

        // For now, we don't have real thread/priority data wired in;
        // they can be added later via ProcessStateManager.
        let threads: Int? = nil
        let priority: Int? = nil

        return Snapshot(
            name: history?.name ?? processName,
            pid: pid,
            lastCpu: lastCpu,
            averageCpu: avgCpu,
            peakCpu: peakCpu,
            loadLevel: loadLevel,
            samples: samples,
            recentValues: recentValues,
            threads: threads,
            priority: priority,
            lastDelta: delta,
            hasData: !samples.isEmpty,
            isStale: isStale
        )
    }

    // MARK: - Body

    var body: some View {
        let snapshot = buildSnapshot()

        VStack(alignment: .leading, spacing: 16) {

            CpuProcessSummaryHeader(
                processName: snapshot.name,
                pid: snapshot.pid,
                lastCpu: snapshot.lastCpu,
                averageCpu: snapshot.averageCpu,
                peakCpu: snapshot.peakCpu,
                loadLevel: snapshot.loadLevel
            )

            HStack(alignment: .top, spacing: 12) {
                ZStack {
                    CpuProcessGraph(samples: snapshot.samples)

                    if !snapshot.hasData {
                        CpuProcessStaleOverlay(kind: .noData)
                            .padding(8)
                    } else if snapshot.isStale {
                        CpuProcessStaleOverlay(kind: .stale)
                            .padding(8)
                    }
                }

                CpuProcessInfoSidebar(
                    threads: snapshot.threads,
                    priority: snapshot.priority,
                    averageCpu: snapshot.averageCpu,
                    peakCpu: snapshot.peakCpu,
                    lastUpdatedDelta: snapshot.lastDelta
                )
                .frame(width: 200)
            }

            VStack(alignment: .leading, spacing: 6) {
                Text("Last few seconds")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                MiniProcessSparkline(values: snapshot.recentValues)
            }
        }
        .padding(12)
    }
}

// MARK: - Preview

#Preview("CpuProcessView") {
    // Fake history for preview only.
    let store = ProcessCpuHistoryStore.shared
    let pid = 1234
    let name = "Xcode"

    let now = Date().timeIntervalSince1970
    for i in 0..<80 {
        let t = now - Double(80 - i)
        let v = 10 + sin(Double(i) / 6.0) * 20 + Double(i % 5)
        store.record(pid: pid, name: name, cpuPercent: max(v, 0), timestamp: t)
    }

    return CpuProcessView(pid: pid, processName: name)
        .environmentObject(ProcessStateManager.shared)
        .frame(width: 640, height: 320)
        .padding()
}
