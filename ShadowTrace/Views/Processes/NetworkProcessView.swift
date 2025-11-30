//
//  NetworkProcessView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Per-process network panel shown inside `ProcessDetailView`.
/// Phase-1: pure UI scaffold with a placeholder graph.
/// Phase-next: wire in per-PID throughput, socket list, remote endpoints, etc.
struct NetworkProcessView: View {
    let pid: Int
    let processName: String

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // HEADER
            Text("Network · \(processName)")
                .font(.headline)

            // DESCRIPTION
            Text("Per-process network throughput, connections, and endpoints will plug in here.")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            // PLACEHOLDER GRAPH AREA
            RoundedRectangle(cornerRadius: 10)
                .strokeBorder(
                    style: StrokeStyle(lineWidth: 1, dash: [4, 4])
                )
                .foregroundStyle(Color.secondary.opacity(0.4))
                .frame(height: 140)
                .overlay(
                    Text("Network graph (Phase-next)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                )

            Spacer()
        }
        .padding(.top, 4)
    }
}

#Preview("Network – Sample Process") {
    NetworkProcessView(
        pid: 1234,
        processName: "Safari"
    )
    .frame(width: 420, height: 240)
    .padding()
}
