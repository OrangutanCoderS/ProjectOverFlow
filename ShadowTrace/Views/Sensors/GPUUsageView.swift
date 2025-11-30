//
//  GPUUsageView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  GPUUsageView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// GPU usage card for the Sensor dashboard.
///
/// Phase-1:
/// - Reads from `GpuUsageStateManager.shared`
/// - Shows current load, VRAM usage, temperature
/// - Simple sparkline of recent usage history
struct GPUUsageView: View {

    @ObservedObject
    private var gpuState = GpuUsageStateManager.shared

    var body: some View {
        let snapshot = buildSnapshot()

        GlassCard {
            VStack(alignment: .leading, spacing: 12) {

                // HEADER
                HStack(spacing: 8) {
                    Image(systemName: "gpu")
                        .font(.system(size: 14, weight: .semibold))
                    Text("GPU")
                        .font(.headline)

                    Spacer()

                    GpuHealthPill(health: snapshot.health)
                }

                // MAIN VALUE
                HStack(alignment: .firstTextBaseline, spacing: 4) {
                    Text(snapshot.usagePercentString)
                        .font(.system(size: 28, weight: .semibold))
                        .monospacedDigit()
                    Text("usage")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                // VRAM + TEMP ROW
                HStack(spacing: 12) {
                    if let vramString = snapshot.vramString {
                        HStack(spacing: 4) {
                            Image(systemName: "memorychip")
                                .font(.caption)
                            Text(vramString)
                                .font(.caption)
                        }
                    }

                    if let tempString = snapshot.temperatureString {
                        HStack(spacing: 4) {
                            Image(systemName: "thermometer.medium")
                                .font(.caption)
                            Text(tempString)
                                .font(.caption)
                        }
                    }

                    Spacer()

                    if let age = snapshot.lastUpdatedAge {
                        Text(age)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }

                // SPARKLINE
                GPUUsageSparkline(values: snapshot.usageSeries)
                    .frame(height: 48)
                    .opacity(snapshot.usageSeries.isEmpty ? 0.35 : 1.0)

                if snapshot.usageSeries.isEmpty {
                    Text("No GPU samples yet. Backend wiring will feed usage here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.top, 2)
                }

                Spacer(minLength: 0)
            }
            .padding(14)
        }
    }

    // MARK: - Snapshot builder

    private func buildSnapshot() -> GpuUsageSnapshot {
        guard let latest = gpuState.latest else {
            return GpuUsageSnapshot.empty
        }

        let series = gpuState.usageSeries(limit: 80)
        let health = GpuHealth.fromUsage(latest.usagePercent)

        return GpuUsageSnapshot(
            usagePercent: latest.usagePercent,
            vramUsedMB: latest.vramUsedMB,
            vramTotalMB: latest.vramTotalMB,
            temperatureC: latest.temperatureC,
            usageSeries: series,
            lastUpdated: latest.timestamp,
            health: health
        )
    }
}

// MARK: - Snapshot view model

private struct GpuUsageSnapshot {
    let usagePercent: Double
    let vramUsedMB: Double?
    let vramTotalMB: Double?
    let temperatureC: Double?
    let usageSeries: [Double]
    let lastUpdated: Date?
    let health: GpuHealth

    static let empty = GpuUsageSnapshot(
        usagePercent: 0,
        vramUsedMB: nil,
        vramTotalMB: nil,
        temperatureC: nil,
        usageSeries: [],
        lastUpdated: nil,
        health: .idle
    )

    var usagePercentString: String {
        String(format: "%.0f%%", usagePercent)
    }

    var vramString: String? {
        guard let used = vramUsedMB else { return nil }
        if let total = vramTotalMB, total > 0 {
            return String(format: "%.0f / %.0f MB", used, total)
        } else {
            return String(format: "%.0f MB used", used)
        }
    }

    var temperatureString: String? {
        guard let temp = temperatureC else { return nil }
        return String(format: "%.0f °C", temp)
    }

    var lastUpdatedAge: String? {
        guard let last = lastUpdated else { return nil }
        let delta = Date().timeIntervalSince(last)

        if delta < 2 { return "Just now" }
        if delta < 60 { return "\(Int(delta))s ago" }
        if delta < 3600 { return "\(Int(delta / 60))m ago" }
        return "\(Int(delta / 3600))h ago"
    }
}

// MARK: - Health enum + pill

private enum GpuHealth {
    case idle
    case normal
    case high
    case saturated

    static func fromUsage(_ usage: Double) -> GpuHealth {
        switch usage {
        case ..<5:    return .idle
        case ..<60:   return .normal
        case ..<90:   return .high
        default:      return .saturated
        }
    }

    var label: String {
        switch self {
        case .idle:      return "Idle"
        case .normal:    return "Normal"
        case .high:      return "High"
        case .saturated: return "Maxed"
        }
    }

    var color: Color {
        switch self {
        case .idle:      return .gray
        case .normal:    return .green
        case .high:      return .orange
        case .saturated: return .red
        }
    }
}

private struct GpuHealthPill: View {
    let health: GpuHealth

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(health.color)
                .frame(width: 8, height: 8)
            Text(health.label)
                .font(.caption2)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule(style: .continuous)
                .fill(health.color.opacity(0.16))
        )
        .foregroundStyle(health.color)
    }
}

// MARK: - Sparkline

private struct GPUUsageSparkline: View {
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
                        Color.cyan.opacity(0.2),
                        Color.cyan
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                style: StrokeStyle(lineWidth: 1.6, lineCap: .round, lineJoin: .round)
            )
        }
    }
}

// MARK: - Preview

#Preview("GPU Usage View") {
    // Seed fake data for preview only
    let manager = GpuUsageStateManager.shared

    let now = Date()
    let fakeSamples = (0..<60).map { i -> GpuUsageSample in
        let t = now.addingTimeInterval(Double(i - 60))
        let usage = Double.random(in: 5...92)
        let used = Double.random(in: 1024...4096)
        let total = 8192.0
        let temp = Double.random(in: 45...78)

        return GpuUsageSample(
            timestamp: t,
            usagePercent: usage,
            vramUsedMB: used,
            vramTotalMB: total,
            temperatureC: temp
        )
    }

    manager.replaceAllSamples(with: fakeSamples)

    return ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        GPUUsageView()
            .frame(width: 360, height: 260)
            .padding(20)
    }
    .environment(\.colorScheme, .dark)
}