//
//  EnergyProcessView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine

/// Per-process energy detail panel.
///
/// Integrates with:
/// - EnergyProcessStateManager.shared
/// - EnergyProcessHistory
/// - EnergyStatsEvent / ThermalImpact (backend models)
struct EnergyProcessView: View {

    let pid: Int
    let processName: String

    @ObservedObject
    private var energyState = EnergyProcessStateManager.shared

    var body: some View {
        let snapshot = buildSnapshot()

        VStack(alignment: .leading, spacing: 16) {

            // HEADER + SUMMARY
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(processName)
                        .font(.headline)
                    Text("PID \(pid)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                VStack(alignment: .trailing, spacing: 4) {
                    HStack(spacing: 6) {
                        Text(String(format: "%.1f", snapshot.energyScore))
                            .font(.system(size: 28, weight: .semibold))
                            .monospacedDigit()
                        Text("energy score")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    HStack(spacing: 8) {
                        ThermalImpactPill(impact: snapshot.thermalImpact)

                        if snapshot.isStale {
                            Text("Stale")
                                .font(.caption2)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(
                                    Capsule()
                                        .fill(Color.orange.opacity(0.15))
                                )
                                .foregroundStyle(.orange)
                        }

                        if let last = snapshot.lastUpdated {
                            Text(Self.relativeTimeString(since: last))
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }

            // GRAPH CARD
            GlassCard {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("Energy over time")
                            .font(.subheadline)
                        Spacer()
                        Text("0 – 100 (normalized)")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }

                    EnergySparkline(values: snapshot.energySeries)
                        .opacity(snapshot.energySeries.isEmpty ? 0.35 : 1.0)
                        .frame(height: 80)

                    if snapshot.energySeries.isEmpty {
                        Text("No energy samples yet for this process.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .padding(.top, 2)
                    }
                }
                .padding(12)
            }

            // METRICS GRID
            EnergyMetricsGrid(snapshot: snapshot)

            Spacer(minLength: 0)
        }
        .padding(.vertical, 8)
    }

    // MARK: - Snapshot

    private func buildSnapshot() -> EnergyProcessSnapshot {
        guard let history = energyState.history(for: pid),
              !history.energyScoreHistory.isEmpty else {
            return EnergyProcessSnapshot.empty(pid: pid, processName: processName)
        }

        let now = Date()
        let isStale = history.isStale(reference: now)

        let series = history.energyScoreHistory

        return EnergyProcessSnapshot(
            pid: pid,
            processName: processName,
            energyScore: history.currentEnergyScore,
            thermalImpact: history.currentThermalImpact,
            cpuContributionPercent: history.currentCpuContributionPercent,
            gpuContributionPercent: history.currentGpuContributionPercent,
            wakeupsPerSec: history.currentWakeupsPerSec,
            estimatedMilliwatts: history.currentMilliwatts,
            peakEnergyScore: history.peakEnergyScore,
            averageEnergyScore: history.averageEnergyScore,
            energySeries: series,
            lastUpdated: history.lastUpdated,
            isStale: isStale
        )
    }

    // MARK: - Time formatting

    private static func relativeTimeString(since date: Date) -> String {
        let delta = Date().timeIntervalSince(date)
        if delta < 2 { return "Just now" }
        if delta < 60 { return "\(Int(delta))s ago" }
        if delta < 3600 { return "\(Int(delta / 60))m ago" }
        return "\(Int(delta / 3600))h ago"
    }
}

// MARK: - Snapshot View Model

private struct EnergyProcessSnapshot {
    let pid: Int
    let processName: String

    let energyScore: Double
    let thermalImpact: ThermalImpact

    let cpuContributionPercent: Double
    let gpuContributionPercent: Double

    let wakeupsPerSec: Int?
    let estimatedMilliwatts: Double?

    let peakEnergyScore: Double
    let averageEnergyScore: Double

    let energySeries: [Double]

    let lastUpdated: Date?
    let isStale: Bool

    static func empty(pid: Int, processName: String) -> EnergyProcessSnapshot {
        EnergyProcessSnapshot(
            pid: pid,
            processName: processName,
            energyScore: 0,
            thermalImpact: .low,
            cpuContributionPercent: 0,
            gpuContributionPercent: 0,
            wakeupsPerSec: nil,
            estimatedMilliwatts: nil,
            peakEnergyScore: 0,
            averageEnergyScore: 0,
            energySeries: [],
            lastUpdated: nil,
            isStale: true
        )
    }
}

// MARK: - Thermal pill

private struct ThermalImpactPill: View {
    let impact: ThermalImpact

    private var label: String {
        switch impact {
        case .low:      return "Low thermal"
        case .medium:   return "Medium thermal"
        case .high:     return "High thermal"
        case .critical: return "Critical thermal"
        }
    }

    private var color: Color {
        switch impact {
        case .low:      return .green
        case .medium:   return .yellow
        case .high:     return .orange
        case .critical: return .red
        }
    }

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(color)
                .frame(width: 8, height: 8)
            Text(label)
                .font(.caption2)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(color.opacity(0.14))
        )
        .foregroundStyle(color)
    }
}

// MARK: - Sparkline

private struct EnergySparkline: View {
    let values: [Double]

    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            let count = max(values.count, 2)
            return Array(repeating: 0.5, count: count)
        }

        return values.map { v in
            let clamped = max(0, min(v, 100))
            let norm = (clamped - minVal) / (maxVal - minVal)
            return CGFloat(max(0, min(norm, 1)))
        }
    }

    var body: some View {
        GeometryReader { geo in
            let w = geo.size.width
            let h = geo.size.height

            let points: [CGPoint] = normalized.enumerated().map { index, n in
                let x = normalized.count > 1
                    ? CGFloat(index) / CGFloat(normalized.count - 1) * w
                    : w / 2
                let y = h - (n * h)
                return CGPoint(x: x, y: y)
            }

            Path { path in
                guard let first = points.first else { return }
                path.move(to: first)
                for p in points.dropFirst() {
                    path.addLine(to: p)
                }
            }
            .stroke(
                LinearGradient(
                    colors: [
                        Color.purple.opacity(0.2),
                        Color.purple
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                style: StrokeStyle(lineWidth: 1.6, lineCap: .round, lineJoin: .round)
            )
        }
    }
}

// MARK: - Metrics Grid

private struct EnergyMetricsGrid: View {
    let snapshot: EnergyProcessSnapshot

    var body: some View {
        HStack(spacing: 12) {

            VStack(spacing: 8) {
                metricCard(
                    title: "CPU contribution",
                    value: String(format: "%.0f%%", snapshot.cpuContributionPercent)
                )
                metricCard(
                    title: "GPU contribution",
                    value: String(format: "%.0f%%", snapshot.gpuContributionPercent)
                )
            }

            VStack(spacing: 8) {
                metricCard(
                    title: "Peak energy",
                    value: String(format: "%.1f", snapshot.peakEnergyScore)
                )
                metricCard(
                    title: "Average energy",
                    value: String(format: "%.1f", snapshot.averageEnergyScore)
                )
            }

            VStack(spacing: 8) {
                metricCard(
                    title: "Wakeups / sec",
                    value: snapshot.wakeupsPerSec.map { "\($0)" } ?? "–"
                )
                metricCard(
                    title: "Estimated power",
                    value: snapshot.estimatedMilliwatts.map { String(format: "%.0f mW", $0) } ?? "–"
                )
            }
        }
    }

    private func metricCard(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption2)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.subheadline)
                .monospacedDigit()
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }
}

// MARK: - Preview

#Preview("Energy Process View") {
    let pid = 1234
    let name = "PreviewApp"

    let manager = EnergyProcessStateManager.shared

    // Fake history – EXACT MATCH to initializer
    let fakeHistory = EnergyProcessHistory(
        pid: pid,
        processName: name,
        energyScoreHistory: (0..<40).map { _ in Double.random(in: 10...90) },
        timestamps: (0..<40).map { i in
            Date().addingTimeInterval(Double(-40 + i))
        },
        currentCpuContributionPercent: 42,
        currentGpuContributionPercent: 12,
        currentThermalImpact: .medium,
        currentWakeupsPerSec: 120,
        currentMilliwatts: 850
    )

    manager.setHistory(fakeHistory, for: pid)

    return EnergyProcessView(
        pid: pid,
        processName: name
    )
    .frame(width: 500, height: 420)
    .padding()
}
