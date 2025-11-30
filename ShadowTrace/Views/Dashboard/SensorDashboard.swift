//
//  SensorDashboard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  SensorDashboard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine

/// High-level sensor dashboard for Phase I.
///
/// Shows a compact overview of:
/// - CPU temperature
/// - GPU temperature (simulated for now)
/// - Fan speed
/// - Battery + thermal state
///
/// Hybrid-ready:
/// - Currently driven by an internal mock generator.
/// - Later you can swap in real sensor feed (SensorsStateManager / backend events)
///   by changing `SensorDashboardViewModel.Mode` implementation only.
struct SensorDashboard: View {

    @StateObject private var model: SensorDashboardViewModel

    // Default init used by the app: auto mode (currently = mock)
    init() {
        _model = StateObject(wrappedValue: SensorDashboardViewModel(mode: .mock))
    }

    // Convenience init for previews/tests to inject a custom model.
    init(model: SensorDashboardViewModel) {
        _model = StateObject(wrappedValue: model)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {

            header

            // GRID OF CARDS
            VStack(spacing: 12) {
                HStack(spacing: 12) {
                    SensorGaugeCard(
                        title: "CPU Temp",
                        value: model.cpuTemperatureString,
                        subtitle: "Cores",
                        series: model.cpuTempHistory,
                        accent: .red
                    )

                    SensorGaugeCard(
                        title: "GPU Temp",
                        value: model.gpuTemperatureString,
                        subtitle: "Die",
                        series: model.gpuTempHistory,
                        accent: .orange
                    )
                }

                HStack(spacing: 12) {
                    SensorGaugeCard(
                        title: "Fan Speed",
                        value: model.fanSpeedString,
                        subtitle: "RPM",
                        series: model.fanSpeedHistory,
                        accent: .blue
                    )

                    BatterySensorCard(
                        levelText: model.batteryLevelString,
                        stateText: model.batteryState.label,
                        thermalText: model.thermalState.label,
                        accent: model.batteryState.color,
                        thermalColor: model.thermalState.color
                    )
                }
            }
        }
        .padding(.vertical, 8)
    }

    // MARK: - Header

    private var header: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Sensors")
                    .font(.headline)

                Text("Live hardware snapshot (Phase I – mock driver)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            HStack(spacing: 6) {
                Circle()
                    .fill(model.isLive ? Color.green : Color.gray)
                    .frame(width: 6, height: 6)
                Text(model.isLive ? "Simulated Live" : "Paused")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

// MARK: - View Model

@MainActor
final class SensorDashboardViewModel: ObservableObject {

    enum Mode {
        case mock
        // case liveBackend  // ← future: wire to SensorsStateManager / backend
    }

    // Published values for UI
    @Published var cpuTempC: Double = 42
    @Published var gpuTempC: Double = 38
    @Published var fanRPM: Int = 1200
    @Published var batteryLevel: Double = 0.82
    @Published var batteryState: BatteryState = .discharging
    @Published var thermalState: ThermalState = .normal

    @Published var cpuTempHistory: [Double] = []
    @Published var gpuTempHistory: [Double] = []
    @Published var fanSpeedHistory: [Double] = []

    @Published var isLive: Bool = true

    private let mode: Mode
    private var timer: Timer?
    private let maxPoints: Int = 60

    init(mode: Mode) {
        self.mode = mode

        switch mode {
        case .mock:
            startMockDriver()
        // case .liveBackend:
        //     // future: subscribe to SensorsStateManager / backend events
        }
    }

    deinit {
        timer?.invalidate()
    }

    // MARK: - Public formatted strings

    var cpuTemperatureString: String {
        String(format: "%.1f °C", cpuTempC)
    }

    var gpuTemperatureString: String {
        String(format: "%.1f °C", gpuTempC)
    }

    var fanSpeedString: String {
        "\(fanRPM) RPM"
    }

    var batteryLevelString: String {
        String(format: "%.0f%%", batteryLevel * 100)
    }

    // MARK: - Mock driver

    private func startMockDriver() {
        isLive = true

        // Seed history with a short ramp so sparkline isn't flat on first frame.
        for i in 0..<20 {
            let t = Double(i) / 20.0
            let cpu = 40 + 10 * sin(t * .pi)
            let gpu = 36 + 8 * sin(t * .pi * 0.9)
            let fan = 1100 + 400 * max(0, sin((t - 0.2) * .pi))

            cpuTempHistory.append(cpu)
            gpuTempHistory.append(gpu)
            fanSpeedHistory.append(fan)
        }

        clampHistory()

        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.stepMock()
            }
        }
        RunLoop.main.add(timer!, forMode: .common)
    }

    private func stepMock() {
        // Simple oscillation + noise
        let cpuDelta = Double.random(in: -0.8...1.2)
        let gpuDelta = Double.random(in: -0.7...1.0)

        cpuTempC = clamp(cpuTempC + cpuDelta, min: 38, max: 88)
        gpuTempC = clamp(gpuTempC + gpuDelta, min: 36, max: 82)

        // Fan reacts to CPU temp
        let targetFan = 1000 + Int((cpuTempC - 40) * 45)  // crude curve
        let fanNoise = Int.random(in: -80...80)
        fanRPM = clamp(targetFan + fanNoise, min: 900, max: 5200)

        // Battery drains slowly; bounce between 25% and 95% to keep UI moving.
        let batteryDelta = Double.random(in: -0.005...0.001)
        batteryLevel = clamp(batteryLevel + batteryDelta, min: 0.25, max: 0.95)

        // Thermal state depends on CPU temp
        switch cpuTempC {
        case ..<65:
            thermalState = .normal
        case ..<80:
            thermalState = .warning
        case ..<90:
            thermalState = .hot
        default:
            thermalState = .critical
        }

        // Battery state: just cycle between discharging / idle / charging.
        let r = Double.random(in: 0...1)
        if r < 0.7 {
            batteryState = .discharging
        } else if r < 0.9 {
            batteryState = .idle
        } else {
            batteryState = .charging
        }

        // Append history points
        cpuTempHistory.append(cpuTempC)
        gpuTempHistory.append(gpuTempC)
        fanSpeedHistory.append(Double(fanRPM))

        clampHistory()
    }

    private func clampHistory() {
        if cpuTempHistory.count > maxPoints {
            let overflow = cpuTempHistory.count - maxPoints
            cpuTempHistory.removeFirst(overflow)
        }
        if gpuTempHistory.count > maxPoints {
            let overflow = gpuTempHistory.count - maxPoints
            gpuTempHistory.removeFirst(overflow)
        }
        if fanSpeedHistory.count > maxPoints {
            let overflow = fanSpeedHistory.count - maxPoints
            fanSpeedHistory.removeFirst(overflow)
        }
    }

    // MARK: - Utils

    private func clamp<T: Comparable>(_ value: T, min: T, max: T) -> T {
        if value < min { return min }
        if value > max { return max }
        return value
    }
}

// MARK: - Battery & Thermal enums

enum BatteryState {
    case charging
    case discharging
    case idle

    var label: String {
        switch self {
        case .charging:     return "Charging"
        case .discharging:  return "On Battery"
        case .idle:         return "Idle"
        }
    }

    var color: Color {
        switch self {
        case .charging:     return .green
        case .discharging:  return .yellow
        case .idle:         return .blue
        }
    }
}

enum ThermalState {
    case normal
    case warning
    case hot
    case critical

    var label: String {
        switch self {
        case .normal:   return "Normal"
        case .warning:  return "Warm"
        case .hot:      return "Hot"
        case .critical: return "Critical"
        }
    }

    var color: Color {
        switch self {
        case .normal:   return .green
        case .warning:  return .yellow
        case .hot:      return .orange
        case .critical: return .red
        }
    }
}

// MARK: - Cards

private struct SensorGaugeCard: View {

    let title: String
    let value: String
    let subtitle: String
    let series: [Double]
    let accent: Color

    var body: some View {
        GlassCard {
            VStack(alignment: .leading, spacing: 8) {

                HStack {
                    Text(title)
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                }

                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text(value)
                        .font(.system(size: 22, weight: .semibold))
                        .monospacedDigit()
                    Text(subtitle)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                SensorSparkline(values: series, accent: accent)
                    .frame(height: 40)
            }
        }
    }
}

private struct BatterySensorCard: View {

    let levelText: String
    let stateText: String
    let thermalText: String
    let accent: Color
    let thermalColor: Color

    var body: some View {
        GlassCard {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Battery & Thermal")
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                }

                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(levelText)
                        .font(.system(size: 22, weight: .semibold))
                        .monospacedDigit()
                    Text(stateText)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                HStack(spacing: 8) {
                    pill(text: stateText, color: accent)
                    pill(text: thermalText, color: thermalColor)
                    Spacer()
                }

                Text("Backend wiring will later replace the mock driver with real sensor stats (SMC / IOKit / OverFlow daemon).")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func pill(text: String, color: Color) -> some View {
        HStack(spacing: 4) {
            Circle()
                .fill(color)
                .frame(width: 6, height: 6)
            Text(text)
                .font(.caption2)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule()
                .fill(color.opacity(0.16))
        )
        .foregroundStyle(color)
    }
}

// MARK: - Sparkline

private struct SensorSparkline: View {
    let values: [Double]
    let accent: Color

    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            let count = max(values.count, 2)
            return Array(repeating: 0.5, count: count)
        }

        return values.map { v in
            let clamped = max(min(v, maxVal), minVal)
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
                        accent.opacity(0.18),
                        accent
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                style: StrokeStyle(lineWidth: 1.4, lineCap: .round, lineJoin: .round)
            )
        }
    }
}

// MARK: - Preview

#Preview("Sensor Dashboard") {
    ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        SensorDashboard(
            model: SensorDashboardViewModel(mode: .mock)
        )
        .frame(width: 620, height: 340)
        .padding(20)
    }
    .environment(\.colorScheme, .dark)
}