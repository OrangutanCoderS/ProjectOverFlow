//
//  DashboardNetworkCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// High-level Network overview card for the main dashboard.
///
/// Shows:
/// - Current total throughput (Up + Down)
/// - Separate labels for upload + download
/// - Simple health pill (Idle / Normal / High)
/// - Sparkline of recent total throughput
struct DashboardNetworkCard: View {

    @ObservedObject private var stats = NetworkStatsAdapter.shared

    var body: some View {
        let snapshot = buildSnapshot()

        GlassCard {
            VStack(alignment: .leading, spacing: 12) {

                // HEADER
                HStack(spacing: 8) {
                    Image(systemName: "network")
                        .font(.system(size: 14, weight: .semibold))
                    Text("Network")
                        .font(.headline)
                    Spacer()
                    NetworkHealthPill(health: snapshot.health)
                }

                // MAIN VALUE
                HStack(alignment: .firstTextBaseline, spacing: 4) {
                    Text(snapshot.totalRateString)
                        .font(.system(size: 28, weight: .semibold))
                        .monospacedDigit()
                    Text("total")
                        .foregroundStyle(.secondary)
                }

                // UP/DOWN ROW
                HStack(spacing: 12) {
                    HStack(spacing: 4) {
                        Image(systemName: "arrow.up")
                            .font(.caption)
                        Text(snapshot.uploadRateString)
                            .font(.subheadline)
                            .monospacedDigit()
                    }

                    HStack(spacing: 4) {
                        Image(systemName: "arrow.down")
                            .font(.caption)
                        Text(snapshot.downloadRateString)
                            .font(.subheadline)
                            .monospacedDigit()
                    }

                    Spacer()

                    if snapshot.activeConnections > 0 {
                        Text("\(snapshot.activeConnections) connections")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                // SPARKLINE
                NetworkSparkline(values: snapshot.totalSeries)

                Spacer(minLength: 0)
            }
            .padding(16)
        }
        .frame(height: 220)
    }
}

// MARK: - Snapshot model
private struct LocalNetworkSnapshot {
    let uploadKBps: Double
    let downloadKBps: Double
    let totalKBps: Double
    let activeConnections: Int
    let droppedPackets: Int
    let totalSeries: [Double]
    let health: NetworkHealth
    let totalRateString: String
    let uploadRateString: String
    let downloadRateString: String
}

// MARK: - Health levels
private enum NetworkHealth {
    case idle
    case normal
    case high

    var label: String {
        switch self {
        case .idle:   return "Idle"
        case .normal: return "Normal"
        case .high:   return "High usage"
        }
    }

    var color: Color {
        switch self {
        case .idle:   return .gray
        case .normal: return .green
        case .high:   return .orange
        }
    }
}

// MARK: - Snapshot builder
private extension DashboardNetworkCard {

    func buildSnapshot() -> LocalNetworkSnapshot {

        guard let s = stats.latestSnapshot else {
            return LocalNetworkSnapshot(
                uploadKBps: 0,
                downloadKBps: 0,
                totalKBps: 0,
                activeConnections: 0,
                droppedPackets: 0,
                totalSeries: [],
                health: .idle,
                totalRateString: "0 KB/s",
                uploadRateString: "0 KB/s",
                downloadRateString: "0 KB/s"
            )
        }

        let upload = max(s.uploadKBps, 0)
        let download = max(s.downloadKBps, 0)
        let total = upload + download

        let health: NetworkHealth
        switch total {
        case ..<1: health = .idle
        case ..<256: health = .normal
        default: health = .high
        }

        let seriesAll = stats.history.map { $0.uploadKBps + $0.downloadKBps }
        let series = Array(seriesAll.suffix(80))

        return LocalNetworkSnapshot(
            uploadKBps: upload,
            downloadKBps: download,
            totalKBps: total,
            activeConnections: s.activeConnections,
            droppedPackets: s.droppedPackets,
            totalSeries: series,
            health: health,
            totalRateString: formatRate(total),
            uploadRateString: formatRate(upload),
            downloadRateString: formatRate(download)
        )
    }
}

// MARK: - Formatting
private func formatRate(_ kbps: Double) -> String {
    if kbps >= 1024 {
        return String(format: "%.1f MB/s", kbps / 1024)
    } else {
        return String(format: "%.0f KB/s", kbps)
    }
}

// MARK: - Health pill
private struct NetworkHealthPill: View {
    let health: NetworkHealth

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(health.color)
                .frame(width: 8, height: 8)
            Text(health.label)
                .font(.caption)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(health.color.opacity(0.12))
        )
    }
}

// MARK: - Sparkline
private struct NetworkSparkline: View {
    let values: [Double]

    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            return Array(repeating: 0.5, count: max(values.count, 2))
        }

        return values.map { CGFloat(($0 - minVal) / (maxVal - minVal)) }
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
            .stroke(
                LinearGradient(
                    colors: [Color.blue.opacity(0.2), Color.blue],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                lineWidth: 1.5
            )
        }
        .frame(height: 40)
    }
}

// MARK: - Preview
#Preview("Network Card") {
    let adapter = NetworkStatsAdapter.shared

    adapter.update(
        NetworkStatsEvent(
            bytesSent: 100_000,
            bytesReceived: 500_000,
            activeConnections: 3,
            droppedPackets: 1
        ),
        timestamp: Date().timeIntervalSince1970
    )

    return DashboardNetworkCard()
        .frame(width: 360)
        .padding()
}
