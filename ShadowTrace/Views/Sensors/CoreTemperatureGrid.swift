//
//  CoreTemperatureGrid.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  CoreTemperatureGrid.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Grid of per-sensor temperatures (CPU, GPU, SSD, PCH, etc.).
struct CoreTemperatureGrid: View {
    let sensors: [ThermalSensorSnapshot]

    private let columns: [GridItem] = [
        GridItem(.flexible(minimum: 90), spacing: 8),
        GridItem(.flexible(minimum: 90), spacing: 8)
    ]

    var body: some View {
        if sensors.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                Text("No thermal sensors yet.")
                    .font(.subheadline)
                Text("Once OverFlow feeds thermal metrics, sensors will appear here.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } else {
            LazyVGrid(columns: columns, spacing: 8) {
                ForEach(sensors) { sensor in
                    sensorCard(sensor)
                }
            }
        }
    }

    private func sensorCard(_ s: ThermalSensorSnapshot) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(s.label)
                .font(.caption)
                .foregroundStyle(.secondary)

            Text(String(format: "%.1f°C", s.currentC))
                .font(.headline)
                .monospacedDigit()

            Text(
                String(
                    format: "Min %.0f°C · Max %.0f°C",
                    s.minC,
                    s.maxC
                )
            )
            .font(.caption2)
            .foregroundStyle(.secondary)
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }
}

#Preview("CoreTemperatureGrid") {
    let sensors = [
        ThermalSensorSnapshot(label: "CPU", currentC: 68, minC: 52, maxC: 80),
        ThermalSensorSnapshot(label: "GPU", currentC: 62, minC: 48, maxC: 76),
        ThermalSensorSnapshot(label: "SSD", currentC: 42, minC: 35, maxC: 50),
        ThermalSensorSnapshot(label: "PCH", currentC: 55, minC: 44, maxC: 64)
    ]

    ZStack {
        Color(NSColor.windowBackgroundColor).ignoresSafeArea()
        GlassCard {
            CoreTemperatureGrid(sensors: sensors)
                .padding(10)
        }
        .padding()
        .environment(\.colorScheme, .dark)
    }
}