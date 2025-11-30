//
//  ProcessTabBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessTabBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Simple segmented tab bar for the per-process inspector.
/// No backend assumptions; just state + visuals.
struct ProcessTabBar: View {

    @Binding var selectedTab: ProcessDetailTab

    var body: some View {
        HStack(spacing: 8) {
            ForEach(ProcessDetailTab.allCases, id: \.self) { tab in
                Button {
                    selectedTab = tab
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: tab.systemImage)
                        Text(tab.title)
                    }
                    .font(.system(size: 11, weight: .medium))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 6)
                    .background(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .fill(
                                selectedTab == tab
                                ? Color.accentColor.opacity(0.12)
                                : Color.clear
                            )
                    )
                    .foregroundStyle(
                        selectedTab == tab ? Color.accentColor : Color.secondary
                    )
                }
                .buttonStyle(.plain)
            }

            Spacer()
        }
    }
}