//
//  CpuProcessInfoSidebar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessInfoSidebar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Right-hand info column showing derived metrics for a process.
struct CpuProcessInfoSidebar: View {

    let threads: Int?
    let priority: Int?
    let averageCpu: Double
    let peakCpu: Double
    let lastUpdatedDelta: TimeInterval?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            infoRow(label: "Average CPU", value: String(format: "%.1f%%", averageCpu))
            infoRow(label: "Peak CPU", value: String(format: "%.1f%%", peakCpu))

            if let threads {
                infoRow(label: "Threads", value: "\(threads)")
            }

            if let priority {
                infoRow(label: "Priority", value: "\(priority)")
            }

            if let delta = lastUpdatedDelta {
                infoRow(label: "Last Sample", value: formatDelta(delta))
            }

            Spacer()
        }
        .font(.caption)
        .padding(10)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }

    private func infoRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value)
                .monospacedDigit()
        }
    }

    private func formatDelta(_ delta: TimeInterval) -> String {
        if delta < 1 { return "Just now" }
        if delta < 60 { return "\(Int(delta))s ago" }
        if delta < 3600 { return "\(Int(delta / 60))m ago" }
        return "\(Int(delta / 3600))h ago"
    }
}