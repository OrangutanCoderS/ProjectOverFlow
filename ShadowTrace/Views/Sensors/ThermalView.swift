//
//  ThermalView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  ThermalView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine

/// High-level thermal overview panel.
///
/// Phase-1.5:
/// - Reads from `ThermalStatsAdapter.shared`
/// - Works even if no backend data yet (shows placeholders)
/// - Ready to be embedded into SensorDashboard or a dedicated tab
struct ThermalView: View {

    @ObservedObject
    private var thermal = ThermalStatsAdapter.shared

    var body: some View {
        let hottest = thermal.hottestSample
        let pressure = thermal.pressureLevel
        let cpu = thermal.cpuTemp
        let gpu = thermal.gpuTemp

        VStack(alignment: .leading, spacing: 16) {

            header(hottest: hottest)

            // TOP ROW: summary metrics + pressure + fan
            HStack(spacing: 12) {
                VStack(spacing: 8) {
                    ThermalMetricCard(
                        title: "CPU",
                        value: cpu.map { String(format: "%.1f°C", $0) } ?? "—",
                        subtitle: cpu == nil ? "No CPU sensor yet" : "Latest CPU temp"
                    )

                    ThermalMetricCard(
                        title: "GPU",
                        value: gpu.map { String(format: "%.1f°C", $0) } ?? "—",
                        subtitle: gpu == nil ? "No GPU sensor yet" : "Latest GPU temp"
                    )
                }

                VStack(spacing: 8) {
                    ThermalPressureCard(level: pressure)

                    FanRPMView(rpm: thermal.fanRPM)
                }
            }

            // GRAPH
            GlassCard {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("Thermal trend")
                            .font(.subheadline)
                        Spacer()
                        if let avg = thermal.averageLast(n: 40) {
                            Text(String(format: "Avg %.1f°C (last 40)", avg))
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }

                    let values = thermal.samples.map { $0.valueC }

                    ThermalMiniGraph(values: values)
                        .opacity(values.isEmpty ? 0.35 : 1.0)

                    if values.isEmpty {
                        Text("No thermal samples yet. Waiting for OverFlow backend.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .padding(12)
            }

            // GRID
            GlassCard {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("Per-sensor temperatures")
                            .font(.subheadline)
                        Spacer()
                    }

                    CoreTemperatureGrid(sensors: thermal.sensorSnapshots)
                }
                .padding(10)
            }

            Spacer(minLength: 0)
        }
        .padding(.vertical, 8)
    }

    // MARK: - Header

    private func header(hottest: ThermalSample?) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Thermal")
                .font(.headline)

            if let hottest {
                Text(String(format: "Hottest sensor: %@ · %.1f°C",
                            hottest.label,
                            hottest.valueC))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                Text("No thermal data yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

// MARK: - Preview

#Preview("ThermalView") {
    let adapter = ThermalStatsAdapter.shared
    adapter.injectPreviewData()

    return ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        ThermalView()
            .frame(width: 640, height: 420)
            .padding(20)
    }
    .environment(\.colorScheme, .dark)
}