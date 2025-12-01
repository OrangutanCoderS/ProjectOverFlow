//  DashboardView.swift
//  ShadowTrace

import SwiftUI

struct DashboardView: View {
    @StateObject private var state = BackendState()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            header

            HStack(alignment: .top, spacing: 16) {
                cpuCard
                memoryCard
            }

            HStack(alignment: .top, spacing: 16) {
                gpuCard
                batteryCard
            }

            Spacer()
        }
        .padding(20)
        .onAppear {
            state.start()
        }
        .onDisappear {
            state.stop()
        }
    }

    // MARK: - Sections

    private var header: some View {
        HStack {
            Text("OverFlow — Live System Dashboard")
                .font(.title2)
                .fontWeight(.semibold)

            Spacer()

            statusBadge
        }
    }

    private var statusBadge: some View {
        Group {
            switch state.connectionStatus {
            case .disconnected:
                Text("Disconnected")
            case .connecting:
                Text("Connecting…")
            case .connected:
                Text("Connected")
            case .error(let msg):
                Text("Error: \(msg)")
            }
        }
        .font(.caption)
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .strokeBorder(lineWidth: 1)
        )
    }

    private var cpuCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("CPU")
                .font(.headline)

            ProgressView(value: state.globalCpu / 100.0) {
                Text(String(format: "Global: %.1f%%", state.globalCpu))
            }
            .progressViewStyle(.linear)

            if !state.perCoreCpu.isEmpty {
                Text("Per Core:")
                    .font(.subheadline)
                    .padding(.top, 4)

                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(Array(state.perCoreCpu.enumerated()), id: \.0) { idx, value in
                            VStack {
                                Text("C\(idx)")
                                    .font(.caption2)
                                Text(String(format: "%.0f%%", value))
                                    .font(.caption)
                            }
                            .padding(6)
                            .background(
                                RoundedRectangle(cornerRadius: 6)
                                    .strokeBorder(lineWidth: 1)
                            )
                        }
                    }
                }
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(NSColor.windowBackgroundColor))
                .shadow(radius: 2)
        )
    }

    private var memoryCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Memory")
                .font(.headline)

            ProgressView(
                value: state.memUsagePercent / 100.0
            ) {
                Text(String(format: "Used: %.1f%%", state.memUsagePercent))
            }
            .progressViewStyle(.linear)

            Text(
                String(
                    format: "%.1f GB / %.1f GB",
                    state.memUsedMB / 1024.0,
                    state.memTotalMB / 1024.0
                )
            )
            .font(.caption)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(NSColor.windowBackgroundColor))
                .shadow(radius: 2)
        )
    }

    private var gpuCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("GPU")
                .font(.headline)

            if !state.gpuName.isEmpty {
                Text(state.gpuName)
                    .font(.subheadline)
            }

            ProgressView(value: state.gpuUsage / 100.0) {
                Text(String(format: "Usage: %.1f%%", state.gpuUsage))
            }
            .progressViewStyle(.linear)

            Text(String(format: "Temp: %.1f ℃", state.gpuTemperature))
                .font(.caption)
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(NSColor.windowBackgroundColor))
                .shadow(radius: 2)
        )
    }

    private var batteryCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Battery")
                .font(.headline)

            ProgressView(value: state.batteryPercentage / 100.0) {
                Text(String(format: "Charge: %.0f%%", state.batteryPercentage))
            }
            .progressViewStyle(.linear)

            HStack(spacing: 8) {
                Circle()
                    .frame(width: 8, height: 8)
                    .foregroundColor(state.batteryCharging ? .green : .orange)
                Text(state.batteryCharging ? "Charging" : "On Battery")
                    .font(.caption)
            }

            if let mins = state.batteryTimeRemainingMin {
                Text(String(format: "Time remaining: %.0f min", mins))
                    .font(.caption2)
            }
        }
        .padding()
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(NSColor.windowBackgroundColor))
                .shadow(radius: 2)
        )
    }
}

// If you want this as the app root:
struct DashboardView_Previews: PreviewProvider {
    static var previews: some View {
        DashboardView()
            .frame(width: 900, height: 500)
    }
}
