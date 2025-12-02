//
//  IORegistrySensorView.swift
//  ShadowTrace
//

import SwiftUI

/// Hierarchical view over hardware sensors grouped by SensorCategory.
/// Backed by SensorsStateManager.shared.sensors.
struct IORegistrySensorView: View {

    @ObservedObject
    private var sensorsState = SensorsStateManager.shared

    /// Which categories are currently expanded.
    @State private var expandedCategories: Set<SensorCategory> = Set(SensorCategory.allCases)

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {

                header

                GlassCard {
                    VStack(alignment: .leading, spacing: 8) {
                        if sensorsState.sensors.isEmpty {
                            Text("No sensor data yet.")
                                .font(.subheadline)
                            Text("Once the backend streams SensorUpdateEvent, live hardware sensors will appear here.")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        } else {
                            ForEach(categoriesSorted) { category in
                                SensorCategorySection(
                                    category: category,
                                    sensors: sensors(in: category),
                                    isExpanded: expandedCategories.contains(category),
                                    toggleExpanded: { toggle(category) }
                                )
                            }
                        }
                    }
                    .padding(12)
                }

                Spacer(minLength: 0)
            }
            .padding(.vertical, 8)
            .padding(.horizontal, 10)
        }
        .background(Color(NSColor.windowBackgroundColor))
    }

    // MARK: - Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Hardware Sensors")
                .font(.title3.bold())
            Text("IORegistry / SMC snapshot (normalized for UI)")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    // MARK: - Category helpers

    private var categoriesSorted: [SensorCategory] {
        let allSensors = sensorsState.sensors
        let used = Set(allSensors.map { $0.category })
        return used.sorted { $0.sortOrder < $1.sortOrder }
    }

    private func sensors(in category: SensorCategory) -> [SensorReading] {
        sensorsState.sensors
            .filter { $0.category == category }
            .sorted { $0.sortOrder < $1.sortOrder }
    }

    private func toggle(_ category: SensorCategory) {
        if expandedCategories.contains(category) {
            expandedCategories.remove(category)
        } else {
            expandedCategories.insert(category)
        }
    }
}

// MARK: - Section view

private struct SensorCategorySection: View {
    let category: SensorCategory
    let sensors: [SensorReading]
    let isExpanded: Bool
    let toggleExpanded: () -> Void

    var body: some View {
        VStack(spacing: 4) {

            Button(action: toggleExpanded) {
                HStack(spacing: 8) {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption2)
                        .foregroundStyle(.secondary)

                    Text(category.title)
                        .font(.subheadline.weight(.semibold))

                    Text("\(sensors.count)")
                        .font(.caption2)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(
                            Capsule().fill(Color(NSColor.controlBackgroundColor))
                        )

                    Spacer()
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                VStack(spacing: 2) {
                    ForEach(sensors) { reading in
                        SensorRowView(reading: reading)
                            .padding(.vertical, 2)
                    }
                }
                .padding(.leading, 18)
            }
        }
    }
}

// MARK: - Sensor row

private struct SensorRowView: View {
    let reading: SensorReading

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(reading.label)
                .font(.caption)
                .frame(minWidth: 180, alignment: .leading)

            Spacer()

            if let v = reading.value {
                Text(String(format: "%.1f %@", v, reading.unit))
                    .font(.caption)
                    .monospacedDigit()
            } else {
                Text("—")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

// MARK: - Preview

#if DEBUG
#Preview("IORegistry Sensors – Empty") {
    ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        IORegistrySensorView()
            .frame(width: 520, height: 380)
            .padding(20)
    }
    .environment(\.colorScheme, .dark)
}

#Preview("IORegistry Sensors – Fake Data") {
    let manager = SensorsStateManager.shared
    manager.replaceAll(with: [
        SensorUpdateEvent(
            key: "TC0P",
            label: "CPU Proximity",
            categoryRaw: "cpu",
            value: 52.3,
            unit: "°C",
            sortOrder: 0
        ),
        SensorUpdateEvent(
            key: "TG0D",
            label: "GPU Diode",
            categoryRaw: "gpu",
            value: 48.7,
            unit: "°C",
            sortOrder: 0
        ),
        SensorUpdateEvent(
            key: "F0Ac",
            label: "Left Fan",
            categoryRaw: "fan",
            value: 2100,
            unit: "RPM",
            sortOrder: 0
        )
    ])

    ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        IORegistrySensorView()
            .frame(width: 520, height: 380)
            .padding(20)
    }
    .environment(\.colorScheme, .dark)
}
#endif
