//
//  CpuProcessGraph.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessGraph.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Canvas-based main CPU graph for a process.
/// Displays the full recent history (up to N samples).
struct CpuProcessGraph: View {

    let samples: [CpuProcessSample]

    var body: some View {
        Canvas { context, size in
            let normalized = CpuProcessGraphRenderer.normalizedYValues(from: samples)
            let points = CpuProcessGraphRenderer.makePoints(values: normalized, size: size)

            guard !points.isEmpty else { return }

            var path = Path()
            path.move(to: points[0])
            for p in points.dropFirst() {
                path.addLine(to: p)
            }

            context.stroke(
                path,
                with: .color(Color.accentColor),
                style: StrokeStyle(lineWidth: 1.6, lineCap: .round, lineJoin: .round)
            )
        }
        .frame(height: 120)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }
}