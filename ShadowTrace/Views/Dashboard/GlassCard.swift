//
//  GlassCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  GlassCard.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import SwiftUI

struct GlassCard<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        content
            .padding(12)
            .background(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(Color(nsColor: NSColor.windowBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(Color.black.opacity(0.08), lineWidth: 1)
            )
            .shadow(color: .black.opacity(0.05), radius: 3, y: 1)
    }
}
