//
//  CpuProcessGraphRenderer.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessGraphRenderer.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Stateless renderer to build a CPU line path from samples.
struct CpuProcessGraphRenderer {

    /// Normalize samples into 0...1 space (Y), preserve order (X).
    static func normalizedYValues(from samples: [CpuProcessSample]) -> [CGFloat] {
        let values = samples.map(\.cpuPercent)
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

    /// Construct points inside a given rect, left-to-right across width.
    static func makePoints(values: [CGFloat], size: CGSize) -> [CGPoint] {
        guard !values.isEmpty else { return [] }

        let width = size.width
        let height = size.height

        let stepX: CGFloat = values.count > 1
            ? width / CGFloat(values.count - 1)
            : 0

        return values.enumerated().map { index, n in
            let x = CGFloat(index) * stepX
            let y = height - (n * height)
            return CGPoint(x: x, y: y)
        }
    }
}