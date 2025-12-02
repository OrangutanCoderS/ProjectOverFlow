//
//  FileActivityView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  FileActivityView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI
import Combine


/// Per-process file activity panel.
///
/// Data source:
/// - FileEventStateManager.shared.events (live feed)
///
/// Scope:
/// - Filters events for the given `pid`
/// - Lets you filter by operation + search by path
/// - Shows a simple inspector for the selected row
struct FileActivityView: View {

    let pid: Int
    let processName: String

    @ObservedObject
    private var fileEventsState = FileEventStateManager.shared

    @State private var searchText: String = ""
    @State private var selectedFilter: FileOpFilter = .all
    @State private var selectedEvent: FileEvent?

    var body: some View {
        let events = filteredEvents

        VStack(alignment: .leading, spacing: 12) {

            // HEADER
            header(eventsCount: events.count)

            // SEARCH + FILTERS
            controls

            // LIST
            GlassCard {
                if events.isEmpty {
                    VStack(spacing: 8) {
                        Text("No file activity yet for this process.")
                            .font(.subheadline)
                        Text("Live events will appear here as OverFlow feeds file I/O into FileEventStateManager.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                } else {
                    ScrollView {
                        LazyVStack(spacing: 4) {
                            ForEach(events, id: \.path) { event in
                                Button {
                                    selectedEvent = event
                                } label: {
                                    FileEventRow(event: event)
                                        .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        .padding(.vertical, 4)
                    }
                    .frame(minHeight: 160, maxHeight: 260)
                }
            }

            // INSPECTOR
            if let event = selectedEvent {
                FileEventInspector(event: event)
            }

            Spacer(minLength: 0)
        }
        .padding(.vertical, 8)
    }

    // MARK: - Derived data

    private var eventsForPid: [FileEvent] {
        fileEventsState.events.filter { $0.pid == pid }
    }

    private var filteredEvents: [FileEvent] {
        var base = eventsForPid

        // Operation filter
        if selectedFilter != .all {
            base = base.filter { $0.opKind == selectedFilter }
        }

        // Search filter
        let trimmed = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty {
            let needle = trimmed.lowercased()
            base = base.filter { $0.path.lowercased().contains(needle) }
        }

        // Most recent first, but we don't assume timestamps exist:
        // we just reverse current ordering.
        return Array(base.suffix(300).reversed())
    }

    // MARK: - Subviews

    private func header(eventsCount: Int) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            VStack(alignment: .leading, spacing: 4) {
                Text("File Activity · \(processName)")
                    .font(.headline)
                Text("\(pid)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            HStack(spacing: 8) {
                Text("\(eventsCount) event\(eventsCount == 1 ? "" : "s")")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                LivePill(isLive: true)
            }
        }
    }

    private var controls: some View {
        FilePathFilterBar(
            searchQuery: $searchText,
            selectedAction: Binding(
                get: {
                    // Convert existing enum → FileAction?
                    selectedFilter == .all ? nil : selectedFilter.asFileAction
                },
                set: { newValue in
                    // Convert FileAction? → existing enum
                    if let a = newValue {
                        selectedFilter = a.asFileOpFilter
                    } else {
                        selectedFilter = .all
                    }
                }
            ),
            onSearchChanged: { _ in /* filtering handled automatically */ },
            onActionChanged: { _ in /* filtering handled automatically */ }
        )
    }
}

// MARK: - Operation filter

/// High-level operation classes for filtering + UI.
enum FileOpFilter: String, CaseIterable, Identifiable {
    case all
    case read
    case write
    case create
    case delete
    case rename
    case modified
    case metadata
    case other

    var id: String { rawValue }

    var label: String {
        switch self {
        case .all:      return "All"
        case .read:     return "Read"
        case .write:    return "Write"
        case .create:   return "Create"
        case .delete:   return "Delete"
        case .rename:   return "Rename"
        case .modified: return "Modified"
        case .metadata: return "Metadata"
        case .other:    return "Other"
        }
    }

    var iconName: String {
        switch self {
        case .all:      return "line.3.horizontal.decrease.circle"
        case .read:     return "eye"
        case .write:    return "pencil"
        case .create:   return "plus"
        case .delete:   return "trash"
        case .rename:   return "arrow.left.and.right"
        case .modified: return "pencil.and.outline"
        case .metadata: return "tag"
        case .other:    return "questionmark"
        }
    }

    var tintColor: Color {
        switch self {
        case .all:      return .secondary
        case .read:     return .blue
        case .write:    return .orange
        case .create:   return .green
        case .delete:   return .red
        case .rename:   return .purple
        case .modified: return .yellow
        case .metadata: return .pink
        case .other:    return .gray
        }
    }
}

private struct FileOpFilterPicker: View {
    @Binding var selected: FileOpFilter

    var body: some View {
        HStack(spacing: 6) {
            ForEach(FileOpFilter.allCases) { filter in
                Button {
                    selected = filter
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: filter.iconName)
                            .font(.caption2)
                        Text(filter.label)
                            .font(.caption)
                    }
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        Capsule(style: .continuous)
                            .fill(selected == filter
                                  ? filter.tintColor.opacity(0.16)
                                  : Color(NSColor.controlBackgroundColor))
                    )
                    .foregroundStyle(selected == filter ? filter.tintColor : .secondary)
                }
                .buttonStyle(.plain)
            }
        }
    }
}


// MARK: - Inspector

private struct FileEventInspector: View {
    let event: FileEvent

    var body: some View {
        GlassCard {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Selected event")
                        .font(.subheadline.weight(.semibold))
                    Spacer()
                    Text(event.opKind.label)
                        .font(.caption)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(
                            Capsule(style: .continuous)
                                .fill(event.opKind.tintColor.opacity(0.16))
                        )
                        .foregroundStyle(event.opKind.tintColor)
                }

                VStack(alignment: .leading, spacing: 4) {
                    labelRow("Path", event.path)
                    labelRow("Operation", event.op)
                    labelRow("Process", "\(event.processName) (\(event.pid))")
                    labelRow("Flagged", event.flagged ? "Yes" : "No")
                }
                .font(.caption)

                Text("Future: show first/last timestamp, total count per path, and anomaly tags from PolicyStateManager.")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .padding(.top, 4)
            }
        }
    }

    private func labelRow(_ label: String, _ value: String) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 6) {
            Text(label)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .frame(width: 80, alignment: .leading)
            Text(value)
                .font(.caption)
                .lineLimit(2)
                .truncationMode(.middle)
            Spacer()
        }
    }
}

// MARK: - Live pill

private struct LivePill: View {
    let isLive: Bool

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(isLive ? Color.green : Color.gray)
                .frame(width: 6, height: 6)
            Text(isLive ? "Live" : "Paused")
                .font(.caption2)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(
            Capsule(style: .continuous)
                .fill((isLive ? Color.green : Color.gray).opacity(0.16))
        )
        .foregroundStyle(isLive ? Color.green : Color.gray)
    }
}

// MARK: - Backend helpers for FileEvent

private extension FileEvent {
    /// Map backend `op` string into our UI enum.
    var opKind: FileOpFilter {
        switch op.lowercased() {
        case "read":   return .read
        case "write":  return .write
        case "create": return .create
        case "delete": return .delete
        case "rename": return .rename
        default:       return .other
        }
    }
}

// MARK: - Mapping FileAction <-> FileOpFilter


private extension FileAction {
    var asFileOpFilter: FileOpFilter {
        switch self {
        case .read:     return .read
        case .write:    return .write
        case .created:  return .create
        case .deleted:  return .delete
        case .renamed:  return .rename
        case .modified: return .modified      // FIXED
        case .metadata: return .metadata      // FIXED
        }
    }
}

private extension FileOpFilter {
    var asFileAction: FileAction? {
        switch self {
        case .all:      return nil
        case .read:     return .read
        case .write:    return .write
        case .create:   return .created
        case .delete:   return .deleted
        case .rename:   return .renamed
        case .modified: return .modified
        case .metadata: return .metadata 
        case .other:    return nil
        }
    }
}

// MARK: - Preview

#Preview("File Activity – Empty") {
    ZStack {
        Color(NSColor.windowBackgroundColor)
            .ignoresSafeArea()

        FileActivityView(
            pid: 4242,
            processName: "PreviewApp"
        )
        .frame(width: 520, height: 380)
        .padding(20)
    }
    .environment(\.colorScheme, .dark)
}
