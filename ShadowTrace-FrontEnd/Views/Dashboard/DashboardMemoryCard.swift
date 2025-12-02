//
//  DashboardMemoryCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  DashboardMemoryCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import SwiftUI

// MARK: - DashboardMemoryCard

struct DashboardMemoryCard: View {
    @ObservedObject private var stats = SystemStatsAdapter.shared

    var body: some View {
        let snapshot = buildSnapshot()

        GlassCard {
            VStack(alignment: .leading, spacing: 12) {

                // HEADER
                Text("Memory")
                    .font(.headline)

                // MAIN VALUE
                HStack(alignment: .firstTextBaseline, spacing: 4) {
                    Text("\(Int(snapshot.percent))%")
                        .font(.system(size: 36, weight: .semibold))
                    Text("used")
                        .foregroundStyle(.secondary)
                }

                // PRESSURE
                HStack(spacing: 6) {
                    Circle()
                        .fill(snapshot.pressure.color)
                        .frame(width: 10, height: 10)

                    Text(snapshot.pressure.rawValue.capitalized)
                        .font(.subheadline)
                        .foregroundColor(snapshot.pressure.color)
                }

                // BREAKDOWN
                if let b = snapshot.breakdown {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Used \(b.usedString) / \(b.totalString)")
                            .font(.subheadline)

                        Text("App: \(b.appString)")
                            .font(.caption)
                            .foregroundStyle(.secondary)

                        if let wired = b.wiredString {
                            Text("Wired: \(wired)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        if let cached = b.cachedString {
                            Text("Cached: \(cached)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }

                // SPARKLINE
                Sparkline(values: snapshot.history)

                Spacer(minLength: 0)
            }
            .padding(16)
        }
        .frame(height: 220)
    }
}

// MARK: - Snapshot Model

private struct MemorySnapshot {
    let used: Double
    let total: Double
    let percent: Double
    let pressure: MemoryPressure
    let breakdown: MemoryBreakdown?
    let history: [Double]
    let lastUpdated: Date
}

private enum MemoryPressure: String {
    case normal
    case warning
    case critical

    var color: Color {
        switch self {
        case .normal: return .green
        case .warning: return .orange
        case .critical: return .red
        }
    }
}

private struct MemoryBreakdown {
    let usedString: String
    let totalString: String
    let appString: String
    let wiredString: String?
    let cachedString: String?
}

// MARK: - Snapshot Builder

private extension DashboardMemoryCard {

    func buildSnapshot() -> MemorySnapshot {

        guard let m = stats.latestMemory else {
            return MemorySnapshot(
                used: 0,
                total: 0,
                percent: 0,
                pressure: .normal,
                breakdown: nil,
                history: [],
                lastUpdated: Date()
            )
        }

        let used = m.usedMb
        let total = max(m.totalMb, 1)
        let percent = (used / total) * 100

        // pressure
        let pressure: MemoryPressure
        switch percent {
        case ..<60: pressure = .normal
        case ..<80: pressure = .warning
        default: pressure = .critical
        }

        let breakdown = MemoryBreakdown(
            usedString: "\(Int(used)) MB",
            totalString: "\(Int(total)) MB",
            appString: "\(Int(max(used - (m.wiredMb ?? 0) - (m.cachedMb ?? 0), 0))) MB",
            wiredString: m.wiredMb != nil ? "\(Int(m.wiredMb!)) MB" : nil,
            cachedString: m.cachedMb != nil ? "\(Int(m.cachedMb!)) MB" : nil
        )

        let history = stats.memoryHistory.map { point in
            let p = (point.usedMb / max(point.totalMb, 1)) * 100
            return min(max(p, 0), 100)
        }

        return MemorySnapshot(
            used: used,
            total: total,
            percent: percent,
            pressure: pressure,
            breakdown: breakdown,
            history: Array(history.suffix(80)), // Enough for sparkline
            lastUpdated: Date()
        )
    }
}

// MARK: - Sparkline

private struct Sparkline: View {
    let values: [Double]

    var normalized: [CGFloat] {
        guard let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            return Array(repeating: 0.5, count: max(values.count, 2))
        }
        return values.map { v in
            CGFloat((v - minVal) / (maxVal - minVal))
        }
    }

    var body: some View {
        GeometryReader { geo in
            let w = geo.size.width
            let h = geo.size.height

            let points = normalized.enumerated().map { (i, n) -> CGPoint in
                let x = CGFloat(i) / CGFloat(max(normalized.count - 1, 1)) * w
                let y = h - (n * h)
                return CGPoint(x: x, y: y)
            }

            Path { path in
                guard let first = points.first else { return }
                path.move(to: first)
                for p in points.dropFirst() { path.addLine(to: p) }
            }
            .stroke(Color.accentColor.opacity(0.8), lineWidth: 2)
        }
        .frame(height: 40)
    }
}

