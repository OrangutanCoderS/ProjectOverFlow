//
//  ProcessMetaView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Metadata tab — Phase-1 (static identity + placeholders)
///
/// Shows:
/// - Basic process identity
/// - Runtime placeholders
/// - Security placeholders
/// - Backend wiring notes
struct ProcessMetaView: View {
    let pid: Int
    let name: String
    let user: String
    let isSystemProcess: Bool

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {

                header

                GlassCard {
                    identitySection
                        .padding(12)
                }

                GlassCard {
                    runtimeSection
                        .padding(12)
                }

                GlassCard {
                    securitySection
                        .padding(12)
                }

                GlassCard {
                    notesSection
                        .padding(12)
                }

                Spacer(minLength: 0)
            }
            .padding(.vertical, 12)
            .padding(.horizontal, 10)
        }
        // IMPORTANT: no infinite frame here
        .background(Color(NSColor.windowBackgroundColor))
    }

    // MARK: - Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name)
                .font(.title3.weight(.semibold))

            Text("PID \(pid) • \(user)")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            Text(isSystemProcess ? "System Process" : "User Process")
                .font(.caption)
                .foregroundStyle(isSystemProcess ? .orange : .green)
        }
    }

    // MARK: - Sections

    private var identitySection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Identity")

            metaRow("Name", name)
            metaRow("PID", "\(pid)")
            metaRow("User", user)
            metaRow("Type", isSystemProcess ? "System" : "User")

            metaRow("Executable", "Not yet available", placeholder: true)
            metaRow("Bundle ID", "Not yet available", placeholder: true)
        }
    }

    private var runtimeSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Runtime")

            metaRow("Start Time", "Not yet available", placeholder: true)
            metaRow("Uptime", "Not yet available", placeholder: true)
            metaRow("Launch Source", "Not yet available", placeholder: true)
        }
    }

    private var securitySection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Security & Sandbox")

            metaRow("Sandboxed", "Not yet available", placeholder: true)
            metaRow("Code Signing", "Not yet available", placeholder: true)
            metaRow("Entitlements", "Not yet available", placeholder: true)
        }
    }

    private var notesSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionTitle("Backend Wiring")

            Text("""
            This tab will later show:
            • Executable path
            • Code signing info
            • Sandbox status
            • Entitlements
            • Container directories
            • LaunchAgent/Daemon sources

            Waiting for backend: ProcessStateManager API extension.
            """)
            .font(.caption)
            .foregroundStyle(.secondary)
        }
    }

    // MARK: - Helpers

    private func sectionTitle(_ title: String) -> some View {
        Text(title.uppercased())
            .font(.caption.weight(.semibold))
            .foregroundStyle(.secondary)
    }

    @ViewBuilder
    private func metaRow(_ label: String, _ value: String, placeholder: Bool = false) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {

            Text(label)
                .font(.caption)
                .foregroundStyle(.secondary)
                .frame(width: 130, alignment: .leading)

            Text(value)
                .font(.subheadline)
                .foregroundStyle(placeholder ? Color.secondary.opacity(0.6) : Color.primary)
                .lineLimit(1)
                .truncationMode(.tail)
                .layoutPriority(1)

            Spacer()
        }
    }
}

// MARK: - Preview

#Preview {
    VStack {
        ProcessMetaView(
            pid: 4242,
            name: "PreviewApp",
            user: "root",
            isSystemProcess: false
        )
        .frame(maxWidth: 420)     // width only
        .padding(20)
    }
    .background(Color(NSColor.windowBackgroundColor))
    .environment(\.colorScheme, .dark)
}
