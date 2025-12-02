//
//  CpuProcessStaleKind.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuProcessStaleOverlay.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

enum CpuProcessStaleKind {
    case noData
    case stale
}

struct CpuProcessStaleOverlay: View {

    let kind: CpuProcessStaleKind

    private var title: String {
        switch kind {
        case .noData: return "No samples yet"
        case .stale:  return "No recent samples"
        }
    }

    private var subtitle: String {
        switch kind {
        case .noData:
            return "Waiting for first CPU event for this process."
        case .stale:
            return "This process has not produced CPU updates recently."
        }
    }

    var body: some View {
        VStack(spacing: 4) {
            Text(title)
                .font(.subheadline)
            Text(subtitle)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(NSColor.windowBackgroundColor).opacity(0.85))
        )
    }
}