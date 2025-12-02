//
//  MemoryProcessView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine

// MARK: - Sample model

private struct MemorySample: Identifiable {
    let id = UUID()
    let timestamp: Date

    let totalMB: Double
    let privateMB: Double?
    let sharedMB: Double?
    let compressedMB: Double?
}

// MARK: - Process-local model

@MainActor
private final class MemoryProcessModel: ObservableObject {

    @Published var samples: [MemorySample] = []

    var latest: MemorySample? { samples.last }

    private let maxSamples = 180    // 3 mins @ 1s
    private var timerCancellable: AnyCancellable?

    private var startTime: Date?
    private var baseMB: Double = 120
    private var jitterMB: Double = 30

    func start(pid: Int, name: String) {
        samples.removeAll()
        startTime = Date()

        let seed = Double(abs(pid % 97))
        baseMB = 80 + seed
        jitterMB = 20 + seed / 2

        timerCancellable?.cancel()

        timerCancellable = Timer
            .publish(every: 1.0, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task { @MainActor in
                    self?.appendSyntheticSample()
                }
            }
    }

    func stop() {
        timerCancellable?.cancel()
        timerCancellable = nil
    }

    private func appendSyntheticSample() {
        let now = Date()
        let t = startTime.map { now.timeIntervalSince($0) } ?? 0

        let wave = sin(t / 18.0) + sin(t / 7.0) * 0.5
        let noise = Double.random(in: -0.4 ... 0.4)

        let total = max(20, baseMB + wave * jitterMB + noise * jitterMB)

        let privateMB = max(0, total * 0.55 + Double.random(in: -5...5))
        let sharedMB  = max(0, total * 0.25 + Double.random(in: -3...3))
        let compressedMB = max(0, total * 0.10 + Double.random(in: -2...2))

        let sample = MemorySample(
            timestamp: now,
            totalMB: total,
            privateMB: privateMB,
            sharedMB: sharedMB,
            compressedMB: compressedMB
        )

        samples.append(sample)

        let overflow = samples.count - maxSamples
        if overflow > 0 { samples.removeFirst(overflow) }
    }
}

// MARK: - Graph

private struct MemoryProcessGraph: View {
    let samples: [MemorySample]

    private struct Point { let x: CGFloat; let y: CGFloat }

    private func normalizedPoints(in size: CGSize) -> [Point] {
        guard samples.count >= 2 else { return [] }

        let vals = samples.map { $0.totalMB }
        guard let minV = vals.min(), let maxV = vals.max(), maxV > minV else {
            let mid = size.height * 0.5
            return [
                Point(x: 0, y: mid),
                Point(x: size.width, y: mid)
            ]
        }

        let dx = size.width / CGFloat(max(samples.count - 1, 1))

        return samples.enumerated().map { idx, s in
            let norm = (s.totalMB - minV) / (maxV - minV)
            return Point(
                x: CGFloat(idx) * dx,
                y: size.height - CGFloat(norm) * size.height
            )
        }
    }

    var body: some View {
        GeometryReader { geo in
            let size = geo.size
            let pts = normalizedPoints(in: size)

            ZStack {
                Path { p in
                    let y = size.height * 0.8
                    p.move(to: .init(x: 0, y: y))
                    p.addLine(to: .init(x: size.width, y: y))
                }
                .stroke(Color.secondary.opacity(0.25), lineWidth: 0.5)

                Path { p in
                    guard let first = pts.first else { return }
                    p.move(to: CGPoint(x: first.x, y: first.y))
                    for pt in pts.dropFirst() {
                        p.addLine(to: CGPoint(x: pt.x, y: pt.y))
                    }
                }
                .stroke(
                    LinearGradient(
                        colors: [
                            Color.accentColor.opacity(0.2),
                            Color.accentColor
                        ],
                        startPoint: .leading,
                        endPoint: .trailing
                    ),
                    style: StrokeStyle(
                        lineWidth: 1.6,
                        lineCap: .round,
                        lineJoin: .round
                    )
                )
            }
        }
        .frame(height: 120)
    }
}

// MARK: - Main View (Public Interface)

struct MemoryProcessView: View {
    let pid: Int
    let processName: String

    @StateObject private var model = MemoryProcessModel()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {

            header

            if model.samples.isEmpty {
                emptyState
            } else {
                MemoryProcessGraph(samples: model.samples)
                    .padding(.vertical, 4)
                breakdown
            }

            Spacer(minLength: 0)
        }
        .onAppear { model.start(pid: pid, name: processName) }
        .onDisappear { model.stop() }
    }

    // MARK: - Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Memory")
                .font(.title2.weight(.semibold))

            Text("\(processName) · PID \(pid)")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            if let latest = model.latest {
                Text(formattedTotal(latest.totalMB))
                    .font(.system(size: 28, weight: .semibold))
                    .monospacedDigit()
            }
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("No samples yet")
                .font(.headline)
            Text("Waiting for memory telemetry…")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    // MARK: - Breakdown

    private var breakdown: some View {
        guard let s = model.latest else { return AnyView(EmptyView()) }

        return AnyView(
            VStack(alignment: .leading, spacing: 6) {
                Text("Breakdown")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                groupRow("Total", formattedMB(s.totalMB))

                if let p = s.privateMB { groupRow("Private", formattedMB(p)) }
                if let sh = s.sharedMB { groupRow("Shared", formattedMB(sh)) }
                if let c = s.compressedMB { groupRow("Compressed", formattedMB(c)) }
            }
            .padding(10)
            .background(
                RoundedRectangle(cornerRadius: 10)
                    .fill(Color(NSColor.controlBackgroundColor))
            )
        )
    }

    private func groupRow(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .font(.caption.weight(.medium))
                .monospacedDigit()
        }
    }

    // MARK: - Formatters

    private func formattedTotal(_ mb: Double) -> String {
        if mb >= 1024 {
            return String(format: "%.1f GB", mb / 1024)
        } else {
            return String(format: "%.0f MB", mb)
        }
    }

    private func formattedMB(_ mb: Double) -> String {
        String(format: "%.0f MB", mb)
    }
}

// MARK: - Preview

#Preview("Memory Process") {
    MemoryProcessView(pid: 1234, processName: "Safari")
        .frame(width: 520, height: 360)
        .padding()
}
