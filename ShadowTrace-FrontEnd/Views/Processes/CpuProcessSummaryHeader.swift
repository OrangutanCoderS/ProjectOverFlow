//
//  CpuProcessSummaryHeader.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessSummaryHeader.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

struct CpuProcessSummaryHeader: View {

    let processName: String?
    let pid: Int
    let lastCpu: Double
    let averageCpu: Double
    let peakCpu: Double
    let loadLevel: CpuLoadLevel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {

            HStack(spacing: 8) {
                Image(systemName: "cpu")
                    .font(.system(size: 16, weight: .semibold))

                Text(processName ?? "PID \(pid)")
                    .font(.system(size: 16, weight: .semibold))

                Text("PID \(pid)")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()

                CpuLoadPill(level: loadLevel)
            }

            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text(String(format: "%.1f%%", lastCpu))
                    .font(.system(size: 34, weight: .bold, design: .rounded))
                    .monospacedDigit()

                VStack(alignment: .leading, spacing: 2) {
                    Text("Avg \(String(format: "%.1f%%", averageCpu))")
                    Text("Peak \(String(format: "%.1f%%", peakCpu))")
                }
                .font(.caption)
                .foregroundStyle(.secondary)

                Spacer()
            }
        }
    }
}

private struct CpuLoadPill: View {
    let level: CpuLoadLevel

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(level.color)
                .frame(width: 8, height: 8)
            Text(level.label)
                .font(.caption)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 4)
        .background(
            Capsule().fill(level.color.opacity(0.12))
        )
    }
}