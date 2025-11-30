//
//  FanRPMView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  FanRPMView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Small card for fan RPM. If RPM is nil, shows "No fan data".
struct FanRPMView: View {
    let rpm: Int?

    var body: some View {
        GlassCard {
            VStack(alignment: .leading, spacing: 6) {
                Text("Fan speed")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let rpm {
                    Text("\(rpm) RPM")
                        .font(.system(size: 20, weight: .semibold))
                        .monospacedDigit()

                    Text("Approximate")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                } else {
                    Text("No fan data yet.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(10)
        }
    }
}

#Preview("FanRPMView") {
    ZStack {
        Color(NSColor.windowBackgroundColor).ignoresSafeArea()
        HStack(spacing: 16) {
            FanRPMView(rpm: 1900)
            FanRPMView(rpm: nil)
        }
        .frame(width: 420)
        .padding()
        .environment(\.colorScheme, .dark)
    }
}