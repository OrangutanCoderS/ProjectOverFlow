//
//  FileEventDetailSheet.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  FileEventDetailSheet.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Foundation

#if os(macOS)
import AppKit
#endif

/// Deep inspector for a single file event.
///
/// Phase-1: works purely off the current FileEvent schema:
///     path, pid, processName, op, flagged
///
/// No assumptions about timestamps or extra backend fields.
/// When the backend grows, we can extend the view without
/// breaking the current API.
struct FileEventDetailSheet: View {

    let event: FileEvent

    // Local, UI-only classification for coloring and labels.
    private var uiAction: FileAction {
        switch event.op.lowercased() {
        case "read":   return .read
        case "write":  return .write
        case "create": return .created
        case "delete": return .deleted
        case "rename": return .renamed
        default:       return .metadata
        }
    }

    @State private var metadata: FileMetadataSnapshot = .placeholder
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        ZStack {
            Color(NSColor.windowBackgroundColor)
                .ignoresSafeArea()

            ScrollView {
                VStack(alignment: .leading, spacing: 16) {

                    headerSection

                    GlassCard {
                        coreInfoSection
                    }

                    GlassCard {
                        pathSection
                    }

                    GlassCard {
                        metadataSection
                    }

                    GlassCard {
                        notesSection
                    }

                    exportAndCloseRow
                }
                .padding(16)
            }
        }
        .onAppear {
            // Synchronous snapshot is fine for Phase-1 scale.
            metadata = FileMetadataSnapshot.load(forPath: event.path)
        }
    }

    // MARK: - Sections

    private var headerSection: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            VStack(alignment: .leading, spacing: 4) {
                Text(event.processName)
                    .font(.title3.weight(.semibold))

                Text("PID \(event.pid)")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            HStack(spacing: 8) {
                Text(uiAction.label)
                    .font(.subheadline.weight(.semibold))

                Circle()
                    .fill(uiActionTint.opacity(0.9))
                    .frame(width: 8, height: 8)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(
                Capsule(style: .continuous)
                    .fill(uiActionTint.opacity(0.18))
            )
            .foregroundStyle(uiActionTint)
        }
    }

    private var coreInfoSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Event")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            infoRow(label: "Operation", value: event.op)
            infoRow(label: "Class", value: uiAction.label)
            infoRow(label: "Flagged", value: event.flagged ? "Yes" : "No")
            infoRow(label: "Timestamp",
                    value: "Not available in Phase-1 log schema")
        }
        .padding(12)
    }

    private var pathSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("File Path")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            Text(event.path)
                .font(.system(.footnote, design: .monospaced))
                .lineLimit(3)
                .truncationMode(.middle)

            HStack(spacing: 8) {
                if metadata.exists {
                    Text("File exists on disk")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else {
                    Text("File not found at this path")
                        .font(.caption)
                        .foregroundStyle(.orange)
                }

                Spacer()

                Button {
                    copyPathToPasteboard(event.path)
                } label: {
                    Label("Copy Path", systemImage: "doc.on.doc")
                        .font(.caption)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
            }
        }
        .padding(12)
    }

    private var metadataSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("File Metadata Snapshot")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            if !metadata.exists {
                Text("No metadata available (file missing or inaccessible).")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                VStack(alignment: .leading, spacing: 4) {
                    infoRow(label: "Type",
                            value: metadata.isDirectory == true ? "Directory" : "Regular file")

                    if let size = metadata.sizeBytes {
                        infoRow(label: "Size",
                                value: formattedSize(size))
                    }

                    if let owner = metadata.owner {
                        infoRow(label: "Owner", value: owner)
                    }

                    if let perms = metadata.permissions {
                        infoRow(label: "Permissions", value: perms)
                    }

                    if let created = metadata.created {
                        infoRow(label: "Created",
                                value: dateFormatter.string(from: created))
                    }

                    if let modified = metadata.modified {
                        infoRow(label: "Modified",
                                value: dateFormatter.string(from: modified))
                    }
                }
            }
        }
        .padding(12)
    }

    private var notesSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Phase-1 Notes")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            Text("""
            This inspector currently reflects:
            • Raw event fields from the FileEvent log stream
            • On-demand filesystem metadata snapshot for the path

            Future phases will enrich this with:
            • Policy matches and anomaly scores
            • Per-path and per-process aggregate stats
            • Temporal correlations with CPU / Disk / Network spikes
            """)
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .padding(12)
    }

    private var exportAndCloseRow: some View {
        HStack {
            Button {
                exportEventAsJSON(event: event, metadata: metadata)
            } label: {
                Label("Export JSON", systemImage: "square.and.arrow.up")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)

            Spacer()

            Button {
                dismiss()
            } label: {
                Text("Close")
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.small)
        }
        .padding(.horizontal, 12)
        .padding(.bottom, 4)
    }

    // MARK: - Helpers

    private var uiActionTint: Color {
        switch uiAction {
        case .read:     return .blue
        case .write:    return .orange
        case .created:  return .green
        case .deleted:  return .red
        case .renamed:  return .purple
        case .modified: return .yellow
        case .metadata: return .pink
        }
    }

    private func infoRow(label: String, value: String) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 6) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .frame(width: 90, alignment: .leading)
            Text(value)
                .font(.caption)
            Spacer()
        }
    }

    private func formattedSize(_ bytes: Int64) -> String {
        if bytes < 1024 {
            return "\(bytes) B"
        }
        let kb = Double(bytes) / 1024.0
        if kb < 1024 {
            return String(format: "%.1f KB", kb)
        }
        let mb = kb / 1024.0
        return String(format: "%.1f MB", mb)
    }

    private var dateFormatter: DateFormatter {
        // Cheap local instance; no performance constraints here.
        let df = DateFormatter()
        df.dateStyle = .medium
        df.timeStyle = .medium
        return df
    }

    private func copyPathToPasteboard(_ path: String) {
        #if os(macOS)
        let pb = NSPasteboard.general
        pb.clearContents()
        pb.setString(path, forType: .string)
        #endif
    }

    private func exportEventAsJSON(event: FileEvent,
                                   metadata: FileMetadataSnapshot) {
        // Phase-1: best-effort export; failures are silently ignored.
        #if os(macOS)
        let fm = FileManager.default

        let exportDir = (fm.homeDirectoryForCurrentUser as NSURL)
            .appendingPathComponent("Documents/ShadowTraceExports", isDirectory: true)!

        try? fm.createDirectory(at: exportDir, withIntermediateDirectories: true)

        let filename = "file_event_\(Int(Date().timeIntervalSince1970)).json"
        let url = exportDir.appendingPathComponent(filename)

        var payload: [String: Any] = [
            "path": event.path,
            "pid": event.pid,
            "processName": event.processName,
            "op": event.op,
            "flagged": event.flagged
        ]

        if metadata.exists {
            payload["metadata"] = metadata.asDictionary()
        }

        if let data = try? JSONSerialization.data(withJSONObject: payload,
                                                  options: [.prettyPrinted]) {
            try? data.write(to: url)
        }
        #endif
    }
}

// MARK: - File metadata snapshot

private struct FileMetadataSnapshot {
    var exists: Bool
    var sizeBytes: Int64?
    var isDirectory: Bool?
    var owner: String?
    var permissions: String?
    var created: Date?
    var modified: Date?

    static let placeholder = FileMetadataSnapshot(
        exists: false,
        sizeBytes: nil,
        isDirectory: nil,
        owner: nil,
        permissions: nil,
        created: nil,
        modified: nil
    )

    static func load(forPath path: String) -> FileMetadataSnapshot {
        let fm = FileManager.default
        var snapshot = FileMetadataSnapshot.placeholder

        var isDir: ObjCBool = false
        let exists = fm.fileExists(atPath: path, isDirectory: &isDir)
        snapshot.exists = exists
        snapshot.isDirectory = exists ? isDir.boolValue : nil

        guard exists else { return snapshot }

        if let attrs = try? fm.attributesOfItem(atPath: path) {
            if let size = attrs[.size] as? NSNumber {
                snapshot.sizeBytes = size.int64Value
            }
            if let owner = attrs[.ownerAccountName] as? String {
                snapshot.owner = owner
            }
            if let perms = attrs[.posixPermissions] as? NSNumber {
                // Octal style like 0755
                snapshot.permissions = String(format: "%o", perms.intValue)
            }
            if let cDate = attrs[.creationDate] as? Date {
                snapshot.created = cDate
            }
            if let mDate = attrs[.modificationDate] as? Date {
                snapshot.modified = mDate
            }
        }

        return snapshot
    }

    func asDictionary() -> [String: Any] {
        var dict: [String: Any] = [
            "exists": exists
        ]
        if let sizeBytes { dict["sizeBytes"] = sizeBytes }
        if let isDirectory { dict["isDirectory"] = isDirectory }
        if let owner { dict["owner"] = owner }
        if let permissions { dict["permissions"] = permissions }
        if let created { dict["created"] = created.timeIntervalSince1970 }
        if let modified { dict["modified"] = modified.timeIntervalSince1970 }
        return dict
    }
}

// MARK: - Preview

#Preview("File Event Detail") {
    let demoEvent = FileEvent(
        path: "/tmp/overflow-demo.log",
        pid: 4242,
        processName: "PreviewApp",
        op: "write",
        flagged: false
    )

    return FileEventDetailSheet(event: demoEvent)
        .frame(width: 520, height: 420)
        .environment(\.colorScheme, .dark)
}