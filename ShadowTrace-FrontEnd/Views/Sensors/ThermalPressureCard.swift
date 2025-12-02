//
//  ThermalPressureCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  ThermalPressureCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Shows overall thermal pressure, based on hottest sensor.
struct ThermalPressureCard: View {
    let level: ThermalPressureLevel

    var body: some View {
        GlassCard {
            HStack(spacing: 10) {
                Text(level.emoji)
                    .font(.system(size: 26))

                VStack(alignment: .leading, spacing: 4) {
                    Text("Thermal pressure")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    Text(level.label)
                        .font(.headline)

                    Text(hint)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                Spacer()
            }
            .padding(10)
        }
    }

    private var hint: String {
        switch level {
        case .cool:
            return "Plenty of thermal headroom."
        case .warm:
            return "Normal operating temperature."
        case .hot:
            return "Device is running warm under load."
        case .critical:
            return "High thermal stress. Throttling likely."
        }
    }
}

#Preview("ThermalPressureCard") {
    ZStack {
        Color(NSColor.windowBackgroundColor).ignoresSafeArea()
        ThermalPressureCard(level: .hot)
            .frame(width: 260)
            .padding()
            .environment(\.colorScheme, .dark)
    }
}