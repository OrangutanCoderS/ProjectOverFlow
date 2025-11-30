//
//  DiskProcessView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Per-process disk I/O panel shown inside `ProcessDetailView`.
///
/// Phase-1 behavior:
///  - If no samples exist -> shows "No disk activity yet".
///  - If samples exist  -> shows:
///     - latest read / write / total (KB/s or MB/s)
///     - cumulative bytes read / written, if known
///     - simple sparkline of total KB/s over time.
struct DiskProcessView: View {
    let pid: Int
    let processName: String

    @ObservedObject private var diskState = DiskProcessStateManager.shared

    var body: some View {
        let history = diskState.history(for: pid)
        let snapshot = DiskSnapshot.from(history: history, fallbackName: processName)

        VStack(alignment: .leading, spacing: 12) {

            // HEADER
            Text("Disk · \(snapshot.processName)")
                .font(.headline)

            if snapshot.hasData {
                // MAIN FIGURES
                HStack(alignment: .firstTextBaseline, spacing: 16) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(snapshot.totalRateString)
                            .font(.system(size: 26, weight: .semibold))
                            .monospacedDigit()
                        Text("Total I/O")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    VStack(alignment: .leading, spacing: 2) {
                        Text("Read: \(snapshot.readRateString)")
                            .font(.subheadline)
                            .monospacedDigit()
                        Text("Write: \(snapshot.writeRateString)")
                            .font(.subheadline)
                            .monospacedDigit()
                    }

                    Spacer()
                }

                // CUMULATIVE BYTES (if available)
                if snapshot.hasCumulative {
                    HStack(spacing: 12) {
                        if let text = snapshot.totalReadBytesString {
                            Label(text, systemImage: "arrow.down.circle")
                                .font(.caption)
                        }
                        if let text = snapshot.totalWrittenBytesString {
                            Label(text, systemImage: "arrow.up.circle")
                                .font(.caption)
                        }
                    }
                    .foregroundStyle(.secondary)
                }

                // SPARKLINE
                RoundedRectangle(cornerRadius: 10)
                    .strokeBorder(Color.secondary.opacity(0.35), lineWidth: 1)
                    .frame(height: 120)
                    .overlay(
                        DiskSparkline(values: snapshot.totalSeries)
                            .padding(.horizontal, 8)
                    )

            } else {
                // NO DATA YET
                Text("No disk activity observed for this process yet.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                RoundedRectangle(cornerRadius: 10)
                    .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
                    .foregroundStyle(Color.secondary.opacity(0.4))
                    .frame(height: 120)
                    .overlay(
                        Text("Disk I/O graph (Phase-next)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    )
            }

            Spacer(minLength: 0)
        }
    }
}

// MARK: - Snapshot

private struct DiskSnapshot {
    let processName: String

    let readKBps: Double
    let writeKBps: Double
    let totalKBps: Double

    let totalReadBytes: Int64?
    let totalWrittenBytes: Int64?

    /// Total KB/s series over time (for sparkline).
    let totalSeries: [Double]

    var hasData: Bool {
        !totalSeries.isEmpty || totalKBps > 0
    }

    var hasCumulative: Bool {
        totalReadBytes != nil || totalWrittenBytes != nil
    }

    // MARK: Formatting

    var readRateString: String { formatRate(readKBps) }
    var writeRateString: String { formatRate(writeKBps) }
    var totalRateString: String { formatRate(totalKBps) }

    var totalReadBytesString: String? {
        guard let bytes = totalReadBytes else { return nil }
        return "Read \(formatBytes(bytes))"
    }

    var totalWrittenBytesString: String? {
        guard let bytes = totalWrittenBytes else { return nil }
        return "Written \(formatBytes(bytes))"
    }

    // MARK: Factory

    static func from(
        history: DiskProcessHistory?,
        fallbackName: String,
        sparklinePoints: Int = 80
    ) -> DiskSnapshot {

        guard let history = history, !history.samples.isEmpty else {
            return DiskSnapshot(
                processName: fallbackName,
                readKBps: 0,
                writeKBps: 0,
                totalKBps: 0,
                totalReadBytes: nil,
                totalWrittenBytes: nil,
                totalSeries: []
            )
        }

        let latest = history.latest!

        let totalSeries = history
            .tail(sparklinePoints)
            .map { max($0.readKBps + $0.writeKBps, 0) }

        return DiskSnapshot(
            processName: history.processName,
            readKBps: max(latest.readKBps, 0),
            writeKBps: max(latest.writeKBps, 0),
            totalKBps: max(latest.readKBps + latest.writeKBps, 0),
            totalReadBytes: latest.totalReadBytes,
            totalWrittenBytes: latest.totalWrittenBytes,
            totalSeries: totalSeries
        )
    }
}

// MARK: - Sparkline

private struct DiskSparkline: View {
    let values: [Double]

    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            let count = max(values.count, 2)
            return Array<CGFloat>(repeating: 0.5, count: count)
        }

        return values.map { v in
            let clamped = max(v, 0)
            let norm = (clamped - minVal) / (maxVal - minVal)
            return CGFloat(min(max(norm, 0), 1))
        }
    }

    var body: some View {
        GeometryReader { geo in
            let w = geo.size.width
            let h = geo.size.height

            let points: [CGPoint] = normalized.enumerated().map { idx, n in
                let x = normalized.count > 1
                    ? CGFloat(idx) / CGFloat(normalized.count - 1) * w
                    : w / 2.0
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
                        Color.purple.opacity(0.18),
                        Color.purple
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                lineWidth: 1.5
            )
        }
    }
}

// MARK: - Formatting helpers

private func formatRate(_ kbps: Double) -> String {
    // kbps here = kilobytes per second
    if kbps >= 1024 {
        let mb = kbps / 1024.0
        return String(format: "%.1f MB/s", mb)
    } else {
        return String(format: "%.0f KB/s", kbps)
    }
}

private func formatBytes(_ bytes: Int64) -> String {
    let kb = Double(bytes) / 1024.0
    if kb < 1024 {
        return String(format: "%.0f KB", kb)
    }
    let mb = kb / 1024.0
    if mb < 1024 {
        return String(format: "%.1f MB", mb)
    }
    let gb = mb / 1024.0
    return String(format: "%.2f GB", gb)
}

// MARK: - Preview

#Preview("Disk Process View") {
    let disk = DiskProcessStateManager.shared
    let pid = 1234
    let name = "PreviewApp"

    // Simulate a small history.
    for i in 0..<40 {
        let t = Date().timeIntervalSince1970 - Double(40 - i)
        let read = Double.random(in: 0...500)
        let write = Double.random(in: 0...300)
        let totalReadBytes = Int64(1_000_000 + i * 50_000)
        let totalWrittenBytes = Int64(500_000 + i * 20_000)

        disk.recordSample(
            for: pid,
            processName: name,
            readKBps: read,
            writeKBps: write,
            totalReadBytes: totalReadBytes,
            totalWrittenBytes: totalWrittenBytes,
            at: t
        )
    }

    return DiskProcessView(pid: pid, processName: name)
        .frame(width: 420, height: 260)
        .padding()
}
