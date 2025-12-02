//
//  ThermalMetricCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  ThermalMetricCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Simple metric card used in the thermal view (e.g. "CPU 68°C").
struct ThermalMetricCard: View {
    let title: String
    let value: String
    let subtitle: String?

    var body: some View {
        GlassCard {
            VStack(alignment: .leading, spacing: 6) {
                Text(title)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Text(value)
                    .font(.system(size: 24, weight: .semibold))
                    .monospacedDigit()

                if let subtitle {
                    Text(subtitle)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(10)
        }
    }
}

#Preview("ThermalMetricCard") {
    ZStack {
        Color(NSColor.windowBackgroundColor).ignoresSafeArea()
        ThermalMetricCard(
            title: "CPU",
            value: "68°C",
            subtitle: "Package temp"
        )
        .frame(width: 140)
        .padding()
        .environment(\.colorScheme, .dark)
    }
}