//
//  ProcessTopBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessTopBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Upper summary strip in the detail view: name, PID, user, quick metrics,
/// and a future "Terminate" action.
struct ProcessTopBar: View {

    let name: String
    let pid: Int
    let user: String
    let isSystemProcess: Bool
    let cpuPercent: Double?
    let memoryMb: Double?
    let onTerminate: (() -> Void)?

    private var cpuLabel: String {
        guard let cpu = cpuPercent else { return "—" }
        return String(format: "%.1f%%", cpu)
    }

    private var memLabel: String {
        guard let mem = memoryMb else { return "—" }
        if mem >= 1024 {
            return String(format: "%.1f GB", mem / 1024.0)
        } else {
            return String(format: "%.0f MB", mem)
        }
    }

    var body: some View {
        HStack(spacing: 12) {

            // Left: name + badges
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(name)
                        .font(.system(size: 18, weight: .semibold))

                    if isSystemProcess {
                        Text("System")
                            .font(.caption2)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(
                                Capsule(style: .continuous)
                                    .fill(Color.gray.opacity(0.15))
                            )
                    }
                }

                HStack(spacing: 10) {
                    Text("PID \(pid)")
                    Text("User: \(user)")
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }

            Spacer()

            // Middle: mini metrics
            HStack(spacing: 16) {
                metricBlock(
                    title: "CPU",
                    value: cpuLabel
                )

                metricBlock(
                    title: "Memory",
                    value: memLabel
                )
            }

            // Right: terminate button (phase-1: no real kill)
            if let onTerminate = onTerminate {
                Button(role: .destructive) {
                    onTerminate()
                } label: {
                    Label("Terminate", systemImage: "xmark.circle")
                }
                .controlSize(.small)
            } else {
                Button {
                    // no-op placeholder
                } label: {
                    Label("Terminate", systemImage: "xmark.circle")
                }
                .controlSize(.small)
                .disabled(true)
                .opacity(0.4)
            }
        }
    }

    // MARK: - Metric block

    private func metricBlock(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title.uppercased())
                .font(.caption2)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.system(size: 14, weight: .medium, design: .monospaced))
        }
    }
}