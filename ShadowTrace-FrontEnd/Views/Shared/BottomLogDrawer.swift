//
//  BottomLogDrawer.swift
//  ShadowTrace
//
//  Created by root_shine on 12/1/25.
//


import SwiftUI

/// Collapsible bottom drawer that shows recent raw JSON lines coming from the Rust backend.
struct BottomLogDrawer: View {
    @EnvironmentObject var backendClient: BackendClient

    @State private var isExpanded: Bool = false
    @State private var maxVisibleLines: Int = 50

    var body: some View {
        VStack(spacing: 0) {
            // Handle / Header
            HStack {
                Capsule()
                    .frame(width: 40, height: 4)
                    .foregroundColor(.secondary)
                    .padding(.top, 4)
                    .padding(.bottom, 4)

                Text("Backend Stream")
                    .font(.caption)
                    .foregroundColor(.secondary)

                Spacer()

                Button(action: { isExpanded.toggle() }) {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.up")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .padding(.trailing, 8)
            }
            .frame(maxWidth: .infinity)
            .contentShape(Rectangle())
            .background(.thinMaterial)
            .onTapGesture {
                isExpanded.toggle()
            }

            if isExpanded {
                Divider()

                // Log content
                ScrollViewReader { proxy in
                    ScrollView(.vertical, showsIndicators: true) {
                        VStack(alignment: .leading, spacing: 2) {
                            ForEach(visibleLines.indices, id: \.self) { idx in
                                Text(visibleLines[idx])
                                    .font(.system(size: 11, weight: .regular, design: .monospaced))
                                    .frame(maxWidth: .infinity, alignment: .leading)
                            }
                        }
                        .padding(8)
                        .id("bottom")
                    }
                    .onChange(of: backendClient.rawLines.count) { _ in
                        // Auto-scroll to bottom when new lines arrive
                        withAnimation {
                            proxy.scrollTo("bottom", anchor: .bottom)
                        }
                    }
                }
                .frame(height: 180)
                .background(Color.black.opacity(0.85))
                .foregroundColor(.green)
            }
        }
        .background(.thinMaterial)
        .cornerRadius(10, corners: [.topLeft, .topRight])
        .shadow(radius: 4)
        .padding(.horizontal, 12)
        .padding(.bottom, 8)
    }

    private var visibleLines: [String] {
        let lines = backendClient.rawLines
        if lines.count <= maxVisibleLines {
            return lines
        } else {
            return Array(lines.suffix(maxVisibleLines))
        }
    }
}

// Helper to round only specific corners
fileprivate extension View {
    func cornerRadius(_ radius: CGFloat, corners: UIRectCorner) -> some View {
        clipShape(RoundedCorner(radius: radius, corners: corners))
    }
}

fileprivate struct RoundedCorner: Shape {
    var radius: CGFloat = 10.0
    var corners: UIRectCorner = [.topLeft, .topRight]

    func path(in rect: CGRect) -> Path {
        let path = UIBezierPath(
            roundedRect: rect,
            byRoundingCorners: corners,
            cornerRadii: CGSize(width: radius, height: radius)
        )
        return Path(path.cgPath)
    }
}