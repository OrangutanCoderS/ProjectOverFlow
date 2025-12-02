//
//  ThermalSample.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  ThermalStatsAdapter.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation
import Combine

/// One temperature reading for a logical sensor (CPU, GPU, SSD, etc.).
struct ThermalSample: Identifiable {
    let id: UUID
    let timestamp: Date
    let label: String       // e.g. "CPU", "GPU", "SSD"
    let valueC: Double      // degrees Celsius

    init(id: UUID = UUID(),
         timestamp: Date,
         label: String,
         valueC: Double) {
        self.id = id
        self.timestamp = timestamp
        self.label = label
        self.valueC = valueC
    }
}

/// Lightweight snapshot for UI grids.
struct ThermalSensorSnapshot: Identifiable {
    let id = UUID()
    let label: String
    let currentC: Double
    let minC: Double
    let maxC: Double
}

/// Overall thermal “pressure” for a simple health pill.
enum ThermalPressureLevel {
    case cool
    case warm
    case hot
    case critical

    var label: String {
        switch self {
        case .cool:     return "Cool"
        case .warm:     return "Warm"
        case .hot:      return "Hot"
        case .critical: return "Critical"
        }
    }

    var emoji: String {
        switch self {
        case .cool:     return "🧊"
        case .warm:     return "🌤"
        case .hot:      return "🔥"
        case .critical: return "☠️"
        }
    }
}

@MainActor
final class ThermalStatsAdapter: ObservableObject {

    static let shared = ThermalStatsAdapter()

    /// Raw samples, newest at the end.
    @Published private(set) var samples: [ThermalSample] = []

    /// Optional fan RPM (backend can set this later).
    @Published private(set) var fanRPM: Int?

    /// Hard cap to avoid unbounded growth.
    private let maxSamples: Int = 300

    private init() {}

    // MARK: - Public API (for future backend wiring)

    /// Append a new thermal sample. Backend can call this per sensor.
    func ingestSample(label: String, valueC: Double, at date: Date = Date()) {
        let sample = ThermalSample(timestamp: date, label: label, valueC: valueC)
        samples.append(sample)
        trimIfNeeded()
    }

    /// Update fan RPM (if known).
    func updateFanRPM(_ rpm: Int?) {
        fanRPM = rpm
    }

    /// Replace all samples (used by previews or test harness).
    func replaceAll(with newSamples: [ThermalSample], fanRPM: Int? = nil) {
        samples = newSamples
        self.fanRPM = fanRPM
        trimIfNeeded()
    }

    // MARK: - Derived metrics

    /// Latest temperature for a given label.
    func latestTemp(for label: String) -> Double? {
        samples
            .last(where: { $0.label == label })
            .map { $0.valueC }
    }

    /// Hottest sample overall.
    var hottestSample: ThermalSample? {
        samples.max(by: { $0.valueC < $1.valueC })
    }

    /// Simple average of the last N samples (all sensors).
    func averageLast(n: Int) -> Double? {
        guard !samples.isEmpty else { return nil }
        let slice = samples.suffix(n)
        let sum = slice.reduce(0.0) { $0 + $1.valueC }
        return sum / Double(slice.count)
    }

    /// CPU and GPU convenience accessors (if we ever feed them).
    var cpuTemp: Double? {
        latestTemp(for: "CPU")
    }

    var gpuTemp: Double? {
        latestTemp(for: "GPU")
    }

    /// Per-sensor min / max / current snapshots.
    var sensorSnapshots: [ThermalSensorSnapshot] {
        let grouped = Dictionary(grouping: samples, by: { $0.label })

        return grouped.keys.sorted().compactMap { key in
            guard let list = grouped[key], !list.isEmpty else { return nil }
            let current = list.last!.valueC
            let minVal = list.map(\.valueC).min() ?? current
            let maxVal = list.map(\.valueC).max() ?? current

            return ThermalSensorSnapshot(
                label: key,
                currentC: current,
                minC: minVal,
                maxC: maxVal
            )
        }
    }

    /// Simple pressure heuristic based on hottest value.
    var pressureLevel: ThermalPressureLevel {
        guard let hottest = hottestSample?.valueC else {
            return .cool
        }

        switch hottest {
        case ..<50:  return .cool
        case ..<70:  return .warm
        case ..<85:  return .hot
        default:     return .critical
        }
    }

    // MARK: - Helpers

    private func trimIfNeeded() {
        let overflow = samples.count - maxSamples
        if overflow > 0 {
            samples.removeFirst(overflow)
        }
    }

    // MARK: - Preview helper

    /// Convenience to seed preview with fake, realistic data.
    func injectPreviewData() {
        var fake: [ThermalSample] = []
        let now = Date()

        let labels = ["CPU", "GPU", "SSD", "PCH"]

        for (idx, label) in labels.enumerated() {
            let base: Double
            switch label {
            case "CPU": base = 65
            case "GPU": base = 60
            case "SSD": base = 40
            case "PCH": base = 50
            default:    base = 55
            }

            for i in 0..<60 {
                let t = now.addingTimeInterval(TimeInterval(-60 + i))
                let jitter = Double.random(in: -3...3)
                let value = base + jitter + Double(idx)
                fake.append(
                    ThermalSample(
                        timestamp: t,
                        label: label,
                        valueC: max(25, value)
                    )
                )
            }
        }

        replaceAll(with: fake, fanRPM: 1800)
    }
}