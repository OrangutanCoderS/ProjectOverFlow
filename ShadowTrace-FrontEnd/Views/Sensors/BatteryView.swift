//
//  BatteryView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  BatteryView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine

/// High-level battery overview card.
///
/// Shows:
/// - Current battery level
/// - Charging / discharging / full status
/// - Simple health classification
/// - Temperature / cycles summary
/// - Sparkline of level history
struct BatteryView: View {

    @ObservedObject
    private var battery = BatteryStatsAdapter.shared

    var body: some View {
        let snapshot = buildSnapshot()

        GlassCard {
            VStack(alignment: .leading, spacing: 12) {

                // HEADER
                HStack(spacing: 8) {
                    Image(systemName: snapshot.iconName)
                        .font(.system(size: 14, weight: .semibold))

                    Text("Battery")
                        .font(.headline)

                    Spacer()

                    BatteryStatusPill(status: snapshot.status)
                }

                // MAIN VALUE
                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text(String(format: "%.0f", snapshot.levelPercent))
                        .font(.system(size: 30, weight: .semibold))
                        .monospacedDigit()

                    Text("%")
                        .font(.title3.weight(.medium))

                    Spacer()

                    if let healthLabel = snapshot.healthLabel {
                        Text(healthLabel)
                            .font(.caption)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(
                                Capsule(style: .continuous)
                                    .fill(snapshot.healthColor.opacity(0.14))
                            )
                            .foregroundStyle(snapshot.healthColor)
                    }
                }

                // SUB ROW: status + temperature + cycles
                HStack(spacing: 12) {

                    HStack(spacing: 4) {
                        Image(systemName: snapshot.statusIconName)
                            .font(.caption)
                        Text(snapshot.statusLabel)
                            .font(.subheadline)
                    }

                    if let temp = snapshot.temperatureC {
                        HStack(spacing: 4) {
                            Image(systemName: "thermometer")
                                .font(.caption2)
                            Text(String(format: "%.0f°C", temp))
                                .font(.caption)
                        }
                        .foregroundStyle(.secondary)
                    }

                    if let cycles = snapshot.cycleCount {
                        HStack(spacing: 4) {
                            Image(systemName: "gobackward")
                                .font(.caption2)
                            Text("\(cycles) cycles")
                                .font(.caption)
                        }
                        .foregroundStyle(.secondary)
                    }

                    Spacer()

                    if let last = snapshot.lastUpdated {
                        Text(Self.relativeTimeString(since: last))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }

                // SPARKLINE
                BatterySparkline(values: snapshot.levelSeries)
                    .frame(height: 44)
                    .opacity(snapshot.levelSeries.isEmpty ? 0.35 : 1.0)

                if snapshot.levelSeries.isEmpty {
                    Text("No battery samples yet. Once the backend feeds BatteryStatsAdapter, history will appear here.")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .padding(.top, 2)
                }

                Spacer(minLength: 0)
            }
            .padding(14)
        }
        .frame(minHeight: 200)
    }

    // MARK: - Snapshot builder

    private func buildSnapshot() -> BatterySnapshot {
        guard let latest = battery.latest else {
            return BatterySnapshot.empty
        }

        let health = classifyHealth(
            healthPercent: latest.healthPercent,
            cycleCount: latest.cycleCount
        )

        let status = classifyStatus(
            level: latest.levelPercent,
            isCharging: latest.isCharging,
            onAC: latest.onACPower
        )

        let series = battery.history.map { $0.levelPercent }

        return BatterySnapshot(
            levelPercent: latest.levelPercent,
            status: status,
            health: health,
            healthPercent: latest.healthPercent,
            cycleCount: latest.cycleCount,
            temperatureC: latest.temperatureC,
            powerWatts: latest.powerWatts,
            isCharging: latest.isCharging,
            onACPower: latest.onACPower,
            levelSeries: series,
            lastUpdated: Date(timeIntervalSince1970: latest.timestamp)
        )
    }

    // MARK: - Classification

    private func classifyStatus(
        level: Double,
        isCharging: Bool,
        onAC: Bool
    ) -> BatteryStatus {
        if level >= 99 && !isCharging {
            return .full
        }
        if isCharging || (onAC && level < 100) {
            return .charging
        }
        return .discharging
    }

    private func classifyHealth(
        healthPercent: Double?,
        cycleCount: Int?
    ) -> BatteryHealth {
        // Very rough, but good enough for Phase-1 UI.
        let hp = healthPercent ?? 100
        let cycles = cycleCount ?? 0

        if hp >= 85 && cycles < 500 {
            return .good
        } else if hp >= 70 && cycles < 1000 {
            return .reduced
        } else {
            return .poor
        }
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

// MARK: - Snapshot ViewModel

private struct BatterySnapshot {
    let levelPercent: Double
    let status: BatteryStatus
    let health: BatteryHealth

    let healthPercent: Double?
    let cycleCount: Int?
    let temperatureC: Double?
    let powerWatts: Double?

    let isCharging: Bool
    let onACPower: Bool

    let levelSeries: [Double]
    let lastUpdated: Date?

    static let empty = BatterySnapshot(
        levelPercent: 0,
        status: .unknown,
        health: .unknown,
        healthPercent: nil,
        cycleCount: nil,
        temperatureC: nil,
        powerWatts: nil,
        isCharging: false,
        onACPower: false,
        levelSeries: [],
        lastUpdated: nil
    )

    // UI helpers

    var statusLabel: String {
        switch status {
        case .charging:    return "Charging"
        case .discharging: return "On battery"
        case .full:        return "Fully charged"
        case .unknown:     return "Unknown"
        }
    }

    var statusIconName: String {
        switch status {
        case .charging:    return "bolt.fill"
        case .discharging: return "battery.25"
        case .full:        return "checkmark.circle"
        case .unknown:     return "questionmark"
        }
    }

    var iconName: String {
        switch levelPercent {
        case 0..<15:   return "battery.0"
        case 15..<35:  return "battery.25"
        case 35..<65:  return "battery.50"
        case 65..<90:  return "battery.75"
        default:       return "battery.100"
        }
    }

    var healthLabel: String? {
        switch health {
        case .good:     return "Health: Good"
        case .reduced:  return "Health: Reduced"
        case .poor:     return "Health: Poor"
        case .unknown:  return nil
        }
    }

    var healthColor: Color {
        switch health {
        case .good:     return .green
        case .reduced:  return .yellow
        case .poor:     return .red
        case .unknown:  return .secondary
        }
    }
}

// MARK: - Enums

private enum BatteryStatus {
    case charging
    case discharging
    case full
    case unknown
}

private enum BatteryHealth {
    case good
    case reduced
    case poor
    case unknown
}

// MARK: - Status pill

private struct BatteryStatusPill: View {
    let status: BatteryStatus

    private var label: String {
        switch status {
        case .charging:    return "Charging"
        case .discharging: return "On battery"
        case .full:        return "Full"
        case .unknown:     return "Unknown"
        }
    }

    private var color: Color {
        switch status {
        case .charging:    return .green
        case .discharging: return .orange
        case .full:        return .blue
        case .unknown:     return .gray
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
            Capsule(style: .continuous)
                .fill(color.opacity(0.16))
        )
        .foregroundStyle(color)
    }
}

// MARK: - Sparkline

private struct BatterySparkline: View {
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
                        Color.green.opacity(0.2),
                        Color.green
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

#Preview("Battery View – Mock") {
    // Seed mock data into the shared adapter.
    let adapter = BatteryStatsAdapter.shared

    let now = Date().timeIntervalSince1970
    var points: [BatteryHistoryPoint] = []

    for i in 0..<40 {
        let levelBase = 100 - Double(i) * 0.7
        let level = max(10, min(levelBase + Double.random(in: -1...1), 100))

        let point = BatteryHistoryPoint(
            timestamp: now - Double(40 - i) * 60,  // 1-min apart
            levelPercent: level,
            isCharging: false,
            onACPower: false,
            healthPercent: 92,
            cycleCount: 120,
            temperatureC: 36 + Double.random(in: -1.5...1.5),
            powerWatts: 7.8
        )
        points.append(point)
    }

    adapter.reset(with: points)

    return ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        BatteryView()
            .frame(width: 360, height: 240)
            .padding(20)
    }
    .environment(\.colorScheme, .dark)
}