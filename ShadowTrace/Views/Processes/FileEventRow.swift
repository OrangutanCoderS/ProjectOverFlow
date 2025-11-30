//
//  FileEventRow.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  FileEventRow.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// One ShadowTrace-styled file activity row.
///
/// - Glass-ish background
/// - Neon outline on hover
/// - Operation-tinted icon
/// - Path compacting to last component
/// - Flag badge when `flagged == true`
struct FileEventRow: View {

    let event: FileEvent

    @State private var isHovering: Bool = false

    // MARK: - Body

    var body: some View {
        HStack(spacing: 10) {

            // ICON
            Image(systemName: iconName)
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(iconColor)
                .frame(width: 22, height: 22)
                .background(
                    Circle()
                        .fill(iconColor.opacity(0.12))
                )

            // TEXT BLOCK
            VStack(alignment: .leading, spacing: 2) {

                // Primary: last path component
                Text(lastPathComponent)
                    .font(.subheadline.weight(.medium))
                    .lineLimit(1)
                    .truncationMode(.tail)

                // Secondary: full path
                Text(event.path)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)

                // Meta line: op + pid + processName
                Text("\(opLabel) • PID \(event.pid) • \(event.processName)")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }

            Spacer(minLength: 8)

            // FLAG PILL
            if event.flagged {
                Text("FLAGGED")
                    .font(.caption2.weight(.semibold))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        Capsule()
                            .fill(Color.red.opacity(0.18))
                    )
                    .foregroundStyle(Color.red)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(rowBackground)
        .overlay(rowStroke)
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.12)) {
                isHovering = hovering
            }
        }
    }

    // MARK: - Derived

    private var lastPathComponent: String {
        URL(fileURLWithPath: event.path).lastPathComponent.isEmpty
        ? event.path
        : URL(fileURLWithPath: event.path).lastPathComponent
    }

    private var opLabel: String {
        switch event.op.lowercased() {
        case "read":   return "Read"
        case "write":  return "Write"
        case "delete": return "Delete"
        case "create": return "Create"
        case "rename": return "Rename"
        default:       return event.op
        }
    }

    private var iconName: String {
        switch event.op.lowercased() {
        case "read":   return "doc.text.magnifyingglass"
        case "write":  return "square.and.pencil"
        case "delete": return "trash"
        case "create": return "plus.rectangle.on.folder"
        case "rename": return "arrow.triangle.2.circlepath"
        default:       return "doc"
        }
    }

    private var iconColor: Color {
        switch event.op.lowercased() {
        case "read":   return .blue
        case "write":  return .purple
        case "delete": return .red
        case "create": return .green
        case "rename": return .orange
        default:       return .gray
        }
    }

    private var rowBackground: some View {
        RoundedRectangle(cornerRadius: 10, style: .continuous)
            .fill(
                Color(NSColor.controlBackgroundColor)
                    .opacity(isHovering ? 0.9 : 0.7)
            )
    }

    private var rowStroke: some View {
        RoundedRectangle(cornerRadius: 10, style: .continuous)
            .stroke(
                LinearGradient(
                    colors: strokeColors,
                    startPoint: .leading,
                    endPoint: .trailing
                ),
                lineWidth: isHovering ? 1.2 : 0.4
            )
            .opacity(isHovering ? 0.95 : 0.35)
    }

    private var strokeColors: [Color] {
        let base = iconColor
        return isHovering
        ? [base.opacity(0.0), base.opacity(0.9)]
        : [base.opacity(0.0), base.opacity(0.45)]
    }
}

// MARK: - Preview

#Preview("File Event Row") {
    VStack(spacing: 8) {
        FileEventRow(
            event: FileEvent(
                path: "/Users/root/Documents/Secrets/plan.md",
                pid: 4242,
                processName: "PreviewApp",
                op: "read",
                flagged: false
            )
        )

        FileEventRow(
            event: FileEvent(
                path: "/System/Library/PrivateStuff/core.db",
                pid: 1337,
                processName: "sneaky-daemon",
                op: "delete",
                flagged: true
            )
        )
    }
    .padding(16)
    .frame(width: 520)
    .background(Color(NSColor.windowBackgroundColor))
    .environment(\.colorScheme, .dark)
}