//
//  DashboardView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import SwiftUI

/// Phase-1 high-level dashboard for OverFlow / ShadowTrace.
/// Pure UI: reads from existing state managers and shows a structured overview.
struct DashboardView: View {

    // MARK: - Global State

    @StateObject private var backendState = BackendStateManager.shared
    @StateObject private var fileEventsState = FileEventStateManager.shared
    @StateObject private var timelineState = TimelineStateManager.shared
    @StateObject private var systemStats = SystemStatsAdapter.shared

    // MARK: - Body

    var body: some View {
        VStack(spacing: 16) {
            headerBar

            ScrollView {
                VStack(spacing: 24) {

                    // SUMMARY
                    summaryRow

                    // SYSTEM METRICS
                    sectionHeader("System Metrics")
                    cpuMemoryRow
                    networkRow

                    // ALERTS
                    alertsSection

                    // TIMELINE
                    timelineSection
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 16)
            }
        }
        .padding(.top, 12)
        .background(
            Color(NSColor.windowBackgroundColor)
                .ignoresSafeArea()
        )
    }

    // MARK: - Header Bar

    private var headerBar: some View {
        HStack(spacing: 12) {

            VStack(alignment: .leading, spacing: 4) {
                Text("OverFlow Dashboard")
                    .font(.system(size: 22, weight: .semibold))

                Text("Phase I – The Eye")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 4) {

                HStack(spacing: 8) {
                    StatusPill(status: backendState.connectionStatus)

                    Text("Backend: \(backendState.backendVersion)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if backendState.inReplayMode {
                    Text("Replay: \(Int(backendState.replayProgress * 100))%")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.horizontal, 16)
    }

    // MARK: - Section Header

    private func sectionHeader(_ title: String) -> some View {
        HStack {
            Text(title)
                .font(.headline)
            Spacer()
        }
        .padding(.horizontal, 4)
    }

    // MARK: - Summary Row

    private var summaryRow: some View {
        HStack(spacing: 12) {
            SummaryCard(
                title: "Connection",
                value: connectionLabel,
                subtitle: "Last event",
                footer: lastEventAgeLabel
            )

            SummaryCard(
                title: "File Activity",
                value: "\(fileEventsState.recentEventsCount)",
                subtitle: "Recent events",
                footer: "Window: \(fileEventsState.recentWindowDescription)"
            )

            SummaryCard(
                title: "Replay",
                value: backendState.inReplayMode ? "Active" : "Idle",
                subtitle: "Progress",
                footer: backendState.inReplayMode
                    ? "\(Int(backendState.replayProgress * 100))% complete"
                    : "Not in replay"
            )
        }
    }

    // MARK: - CPU + Memory + Network row

    private var cpuMemoryRow: some View {
        HStack(spacing: 16) {

            DashboardCpuCard()
                .frame(maxWidth: .infinity, minHeight: 180)
                .opacity(systemStats.latestCpu == nil ? 0.4 : 1.0)
                .animation(.easeInOut(duration: 0.15), value: systemStats.latestCpu != nil)

            DashboardMemoryCard()
                .frame(maxWidth: .infinity, minHeight: 180)
                .opacity(systemStats.latestMemory == nil ? 0.4 : 1.0)
                .animation(.easeInOut(duration: 0.15), value: systemStats.latestMemory != nil)
        }
    }
    // MARK: - Network Row

    private var networkRow: some View {
        HStack(spacing: 16) {
            DashboardNetworkCard()
                .frame(maxWidth: .infinity, minHeight: 220)
        }
    }

    // MARK: - Alerts Section

    private var alertsSection: some View {
        VStack(alignment: .leading, spacing: 8) {

            HStack {
                Text("Global Alerts")
                    .font(.headline)
                Spacer()

                if !backendState.globalAlerts.isEmpty {
                    Button("Clear All") {
                        backendState.clearAllAlerts()
                    }
                    .font(.caption)
                    .buttonStyle(.borderless)
                }
            }

            if backendState.globalAlerts.isEmpty {
                Text("No active alerts.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 4)
            } else {
                VStack(spacing: 4) {
                    ForEach(backendState.globalAlerts) { alert in
                        AlertRow(alert: alert)
                    }
                }
            }
        }
    }

    // MARK: - Timeline Section

    private var timelineSection: some View {
        VStack(alignment: .leading, spacing: 8) {

            HStack {
                Text("Timeline")
                    .font(.headline)
                Spacer()
                Text(backendState.inReplayMode ? "Replay Mode" : "Live")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            RoundedRectangle(cornerRadius: 10)
                .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
                .foregroundStyle(Color.secondary.opacity(0.4))
                .frame(height: 80)
                .overlay(
                    VStack(spacing: 4) {
                        Text("Timeline Overview")
                            .font(.subheadline)
                        Text("GraphView integration will plug in here (Phase-next).")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                )
        }
    }

    // MARK: - Derived Labels

    private var connectionLabel: String {
        switch backendState.connectionStatus {
        case .disconnected: return "Disconnected"
        case .connecting:   return "Connecting"
        case .connected:    return "Connected"
        case .error:        return "Error"
        }
    }

    private var lastEventAgeLabel: String {
        let ts = backendState.lastEventTimestamp
        guard ts > 0 else { return "No events yet" }

        let delta = Date().timeIntervalSince1970 - ts

        if delta < 2 { return "Just now" }
        if delta < 60 { return "\(Int(delta))s ago" }
        if delta < 3600 { return "\(Int(delta / 60))m ago" }
        return "\(Int(delta / 3600))h ago"
    }
}

// MARK: - FileEventStateManager convenience

private extension FileEventStateManager {
    var recentEventsCount: Int { events.count }
    var recentWindowDescription: String { "current session" }
}

// MARK: - Supporting Views

private struct StatusPill: View {
    let status: ConnectionStatus

    private var label: String {
        switch status {
        case .disconnected: return "Disconnected"
        case .connecting:   return "Connecting"
        case .connected:    return "Connected"
        case .error:        return "Error"
        }
    }

    private var color: Color {
        switch status {
        case .disconnected: return .gray
        case .connecting:   return .orange
        case .connected:    return .green
        case .error:        return .red
        }
    }

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(color)
                .frame(width: 8, height: 8)
            Text(label)
                .font(.caption)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(Capsule().fill(color.opacity(0.12)))
    }
}

private struct SummaryCard: View {
    let title: String
    let value: String
    let subtitle: String
    let footer: String

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {

            Text(title)
                .font(.caption)
                .foregroundStyle(.secondary)

            Text(value)
                .font(.system(size: 22, weight: .semibold))

            Spacer(minLength: 4)

            Text(subtitle)
                .font(.caption2)
                .foregroundStyle(.secondary)

            Text(footer)
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(12)
        .frame(maxWidth: .infinity, minHeight: 90, alignment: .topLeading)
        .background(
            RoundedRectangle(cornerRadius: 10)
                .fill(Color(NSColor.controlBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(Color(NSColor.separatorColor), lineWidth: 0.5)
        )
    }
}

private struct AlertRow: View {
    let alert: AlertItem

    private var iconColor: Color {
        switch alert.level {
        case .info:     return .blue
        case .warning:  return .orange
        case .error:    return .red
        case .critical: return .red
        }
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {

            Circle()
                .fill(iconColor)
                .frame(width: 6, height: 6)
                .padding(.top, 6)

            VStack(alignment: .leading, spacing: 2) {
                Text(alert.title)
                    .font(.subheadline)
                Text(alert.message)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            Text(Self.format(date: alert.timestamp))
                .font(.caption2)
                .foregroundStyle(.secondary)
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }

    private static func format(date: Date) -> String {
        let f = DateFormatter()
        f.timeStyle = .short
        f.dateStyle = .none
        return f.string(from: date)
    }
}
