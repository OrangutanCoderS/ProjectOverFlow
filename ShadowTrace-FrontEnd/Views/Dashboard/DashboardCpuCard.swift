//
//  DashboardCpuCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import SwiftUI

/// High-level CPU overview card for the main dashboard.
/// Shows:
/// - Total CPU usage
/// - Lightweight sparkline of recent usage
/// - Optional per-core mini grid
struct DashboardCpuCard: View {

    // MARK: - Local Types

    private enum ViewMode: String, CaseIterable, Identifiable {
        case aggregate
        case perCoreGrid

        var id: String { rawValue }

        var label: String {
            switch self {
            case .aggregate:   return "Overview"
            case .perCoreGrid: return "Per-Core"
            }
        }

        var systemImage: String {
            switch self {
            case .aggregate:   return "waveform.path.ecg"
            case .perCoreGrid: return "square.grid.2x2"
            }
        }
    }

    private enum CoreFilter: String, CaseIterable, Identifiable {
        case all
        case efficiency
        case performance

        var id: String { rawValue }

        var label: String {
            switch self {
            case .all:         return "All"
            case .efficiency:  return "E-cores"
            case .performance: return "P-cores"
            }
        }
    }

    // MARK: - State

    @ObservedObject private var stats = SystemStatsAdapter.shared

    @State private var viewMode: ViewMode = .aggregate
    @State private var coreFilter: CoreFilter = .all

    // MARK: - Derived Labels

    private var totalCpuPercent: Double {
        guard let latest = stats.latestCpu else { return 0 }
        let value = latest.userPercent + latest.systemPercent
        return min(max(value, 0), 100)
    }

    private var cpuPercentText: String {
        String(format: "%.1f%%", totalCpuPercent)
    }

    private var relativeUpdatedString: String {
        guard let lastTs = stats.cpuHistory.last?.timestamp else {
            return "Waiting for data…"
        }
        let delta = Date().timeIntervalSince1970 - lastTs
        if delta < 5 {
            return "Updated just now"
        } else if delta < 60 {
            return "Updated \(Int(delta))s ago"
        } else {
            let minutes = Int(delta / 60)
            return "Updated \(minutes)m ago"
        }
    }

    /// Last ~60 points of total CPU usage for sparkline.
    private var recentCpuSeries: [Double] {
        let history = stats.cpuHistory
        guard !history.isEmpty else { return [] }
        let tail = history.suffix(60)
        return tail.map { $0.userPercent + $0.systemPercent }
    }

    // MARK: - Body

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            header
            modeToggle

            switch viewMode {
            case .aggregate:
                aggregateSection
            case .perCoreGrid:
                perCoreSection
            }
        }
        .padding(14)
        .background(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(Color(nsColor: .windowBackgroundColor))
                .shadow(color: Color.black.opacity(0.12), radius: 8, x: 0, y: 4)
        )
    }

    // MARK: - Sections

    private var header: some View {
        HStack(spacing: 8) {
            Image(systemName: "cpu")
                .font(.system(size: 16, weight: .semibold))
            Text("CPU Usage")
                .font(.system(size: 14, weight: .semibold))
            Spacer()
            Text(relativeUpdatedString)
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
        }
    }

    private var modeToggle: some View {
        HStack(spacing: 8) {
            Picker("", selection: $viewMode) {
                ForEach(ViewMode.allCases) { mode in
                    Label(mode.label, systemImage: mode.systemImage)
                        .tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()

            if viewMode == .perCoreGrid {
                Picker("", selection: $coreFilter) {
                    ForEach(CoreFilter.allCases) { filter in
                        Text(filter.label).tag(filter)
                    }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .frame(maxWidth: 220)
            }

            Spacer()
        }
        .font(.system(size: 11))
    }

    private var aggregateSection: some View {
        HStack(alignment: .center, spacing: 16) {
            VStack(alignment: .leading, spacing: 4) {
                Text(cpuPercentText)
                    .font(.system(size: 32, weight: .bold, design: .rounded))
                    .monospacedDigit()

                if let latest = stats.latestCpu {
                    HStack(spacing: 8) {
                        pill("User \(Int(latest.userPercent))%", color: .accentColor)
                        pill("System \(Int(latest.systemPercent))%", color: .orange)
                        pill("Idle \(Int(latest.idlePercent))%", color: .secondary)
                    }
                    .font(.system(size: 11, weight: .medium))
                } else {
                    Text("No samples yet")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                }
            }

            Spacer(minLength: 0)

            SparklineView(values: recentCpuSeries)
                .frame(width: 140, height: 40)
        }
    }

    private var perCoreSection: some View {
        let cores = filteredPerCoreValues()

        return VStack(alignment: .leading, spacing: 8) {
            if cores.isEmpty {
                Text("No per-core data available")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            } else {
                LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 6), count: 4),
                          spacing: 6) {
                    ForEach(Array(cores.enumerated()), id: \.offset) { (idx, value) in
                        CoreBarView(coreIndex: idx, value: value)
                    }
                }
            }
        }
    }

    // MARK: - Helpers

    private func pill(_ text: String, color: Color) -> some View {
        Text(text)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(
                Capsule(style: .continuous)
                    .fill(color.opacity(0.12))
            )
            .foregroundStyle(color)
    }

    /// Returns per-core usage values filtered according to `coreFilter`.
    /// We approximate:
    /// - First half of cores = "efficiency"
    /// - Second half of cores = "performance"
    private func filteredPerCoreValues() -> [Double] {
        guard let latest = stats.latestCpu else { return [] }
        let all = latest.perCore
        guard !all.isEmpty else { return [] }

        switch coreFilter {
        case .all:
            return all
        case .efficiency:
            let mid = max(all.count / 2, 1)
            return Array(all.prefix(mid))
        case .performance:
            let mid = max(all.count / 2, 1)
            return Array(all.suffix(from: mid))
        }
    }
}

// MARK: - Lightweight sparkline

private struct SparklineView: View {
    let values: [Double]

    /// Normalized [0, 1] as CGFloats.
    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            // Flat line at 0.5 if no variance / no data.
            let count = max(values.count, 2)
            return Array<CGFloat>(repeating: 0.5, count: count)
        }

        return values.map { value in
            let norm = (value - minVal) / (maxVal - minVal)
            return CGFloat(norm.clamped(to: 0...1))
        }
    }

    private func makePoints(width: CGFloat, height: CGFloat) -> [CGPoint] {
        let samples = normalized
        guard !samples.isEmpty else { return [] }

        let stepX: CGFloat = samples.count > 1
            ? width / CGFloat(samples.count - 1)
            : 0

        return samples.enumerated().map { (index, v) in
            let x = CGFloat(index) * stepX
            let y = height - (v * height)
            return CGPoint(x: x, y: y)
        }
    }

    var body: some View {
        GeometryReader { geo in
            let width = geo.size.width
            let height = geo.size.height
            let points = makePoints(width: width, height: height)

            Path { path in
                guard let first = points.first else { return }
                path.move(to: first)
                for p in points.dropFirst() {
                    path.addLine(to: p)
                }
            }
            .stroke(style: StrokeStyle(lineWidth: 1.4, lineCap: .round, lineJoin: .round))
            .foregroundStyle(
                LinearGradient(
                    colors: [Color.accentColor.opacity(0.15), Color.accentColor],
                    startPoint: .leading,
                    endPoint: .trailing
                )
            )
        }
    }
}

// MARK: - Per-core mini bar

private struct CoreBarView: View {
    let coreIndex: Int
    let value: Double   // expected 0-100

    private var clampedValue: Double {
        max(0, min(value, 100))
    }

    var body: some View {
        VStack(spacing: 4) {
            GeometryReader { geo in
                let height = geo.size.height
                let filled = height * CGFloat(clampedValue / 100.0)

                ZStack(alignment: .bottom) {
                    RoundedRectangle(cornerRadius: 4, style: .continuous)
                        .fill(Color.secondary.opacity(0.12))

                    RoundedRectangle(cornerRadius: 4, style: .continuous)
                        .fill(Color.accentColor)
                        .frame(height: filled)
                }
            }
            .frame(height: 32)

            Text("C\(coreIndex)")
                .font(.system(size: 9, weight: .medium, design: .monospaced))
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Utility

private extension Double {
    func clamped(to range: ClosedRange<Double>) -> Double {
        min(max(self, range.lowerBound), range.upperBound)
    }
}

// MARK: - Preview

#Preview("CPU Card") {
    do {
        let adapter = SystemStatsAdapter.shared
        adapter.updateCpu(
            CpuStatsEvent(
                userPercent: 23.5,
                systemPercent: 12.0,
                idlePercent: 64.5,
                perCore: [10, 20, 30, 40, 55, 65, 70, 80]
            )
        )
    }

    return DashboardCpuCard()
        .frame(width: 360)
        .padding()
}
