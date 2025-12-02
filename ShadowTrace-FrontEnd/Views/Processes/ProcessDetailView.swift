//
//  ProcessDetailView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessDetailView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Full-screen inspector for a single process.
/// Phase-1: purely presentational. You pass in the summary
/// values from `ProcessListView` (no hard binding to backend).
struct ProcessDetailView: View {

    // MARK: - Identity

    let pid: Int
    let name: String
    let user: String
    let isSystemProcess: Bool

    /// Optional summary metrics at the time of navigation.
    /// These are *snapshotted* values – they don't have to live-update yet.
    let cpuPercent: Double?
    let memoryMb: Double?

    // MARK: - Local UI State

    @State private var selectedTab: ProcessDetailTab = .cpu

    // MARK: - Body

    var body: some View {
        VStack(spacing: 0) {

            ProcessTopBar(
                name: name,
                pid: pid,
                user: user,
                isSystemProcess: isSystemProcess,
                cpuPercent: cpuPercent,
                memoryMb: memoryMb,
                onTerminate: nil      // Wire to backend later
            )
            .padding(.horizontal, 16)
            .padding(.vertical, 10)

            Divider()

            ProcessTabBar(selectedTab: $selectedTab)
                .padding(.horizontal, 16)
                .padding(.top, 6)
                .padding(.bottom, 4)

            Divider()

            // Tab content
            tabContent
                .padding(16)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .navigationTitle(name)
        .navigationSubtitle("PID \(pid) • \(user)")
    }

    // MARK: - Tab Content

    @ViewBuilder
    private var tabContent: some View {
        switch selectedTab {
        case .cpu:
            CpuProcessView(pid: pid, processName: name)

        case .memory:
            MemoryProcessView(pid: pid, processName: name)

        case .disk:
            DiskProcessView(pid: pid, processName: name)

        case .network:
            NetworkProcessView(pid: pid, processName: name)

        case .energy:
            EnergyProcessView(pid: pid, processName: name)

        case .meta:
            ProcessMetaView(
                pid: pid,
                name: name,
                user: user,
                isSystemProcess: isSystemProcess
            )
        }
    }
}

// MARK: - Preview

#Preview("Process Detail") {
    NavigationStack {
        ProcessDetailView(
            pid: 1234,
            name: "Safari",
            user: "root_shine",
            isSystemProcess: false,
            cpuPercent: 12.4,
            memoryMb: 512
        )
    }
    .frame(width: 900, height: 520)
}
