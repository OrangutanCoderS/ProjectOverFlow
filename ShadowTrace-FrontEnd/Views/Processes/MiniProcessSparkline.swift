//
//  MiniProcessSparkline.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  MiniProcessSparkline.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Tiny sparkline for "last few seconds" view of CPU.
struct MiniProcessSparkline: View {

    let values: [Double]

    private var normalized: [CGFloat] {
        guard !values.isEmpty,
              let minVal = values.min(),
              let maxVal = values.max(),
              maxVal > minVal else {
            let count = max(values.count, 2)
            return Array(repeating: 0.5, count: count)
        }

        return values.map { v in
            let n = (v - minVal) / (maxVal - minVal)
            return CGFloat(min(max(n, 0), 1))
        }
    }

    var body: some View {
        GeometryReader { geo in
            let w = geo.size.width
            let h = geo.size.height

            let points: [CGPoint] = normalized.enumerated().map { index, n in
                let x = normalized.count > 1
                    ? CGFloat(index) / CGFloat(normalized.count - 1) * w
                    : w / 2
                let y = h - (n * h)
                return CGPoint(x: x, y: y)
            }

            Path { path in
                guard let first = points.first else { return }
                path.move(to: first)
                for p in points.dropFirst() { path.addLine(to: p) }
            }
            .stroke(
                LinearGradient(
                    colors: [
                        Color.accentColor.opacity(0.18),
                        Color.accentColor
                    ],
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                lineWidth: 1.2
            )
        }
        .frame(height: 28)
    }
}