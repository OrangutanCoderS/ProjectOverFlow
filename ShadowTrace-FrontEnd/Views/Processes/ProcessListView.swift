//
//  ProcessListView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

// MARK: - View-Model

/// Lightweight view-model for a single process row.
/// This is intentionally decoupled from backend structs so the UI
/// stays stable even if the Rust schema changes.
struct ProcessRowModel: Identifiable, Hashable {
    let id: Int          // PID as stable identifier
    let name: String
    let user: String
    let cpuPercent: Double
    let memoryMb: Double
    let isSystem: Bool

    /// Optional: you can later add fields like `commandLine`, `flags`, etc.
}

// MARK: - Sort / Column Models

private enum ProcessColumn: Hashable {
    case name
    case pid
    case user
    case cpu
    case memory

    var title: String {
        switch self {
        case .name:   return "Process"
        case .pid:    return "PID"
        case .user:   return "User"
        case .cpu:    return "CPU %"
        case .memory: return "Memory (MB)"
        }
    }

    /// Right-align numeric columns.
    var alignment: Alignment {
        switch self {
        case .cpu, .memory, .pid:
            return .trailing
        default:
            return .leading
        }
    }

    /// Width hints; these keep the table from wobbling.
    var minWidth: CGFloat {
        switch self {
        case .name:   return 220
        case .pid:    return 60
        case .user:   return 120
        case .cpu:    return 80
        case .memory: return 110
        }
    }
}

// MARK: - Main Process List View

/// High-level process table for the Processes page.
/// This version is *pure UI* and expects an array of `ProcessRowModel`.
/// Later you’ll feed this from ProcessStateManager / ProcessStatsAdapter.
struct ProcessListView: View {

    // MARK: - Inputs

    /// Source rows (e.g. mapped from your ProcessStateManager).
    let rows: [ProcessRowModel]

    // MARK: - Local UI State

    @State private var sortColumn: ProcessColumn = .cpu
    @State private var sortAscending: Bool = false

    @State private var searchText: String = ""
    @State private var showSystemProcesses: Bool = true

    // Later you can persist pinned PIDs in a state manager if you want.
    @State private var pinnedPids: Set<Int> = []

    // MARK: - Derived Collections

    private var filteredAndSortedRows: [ProcessRowModel] {
        var result = rows

        // 1) Filter: system vs user processes
        if !showSystemProcesses {
            result = result.filter { !$0.isSystem }
        }

        // 2) Filter: search text
        let trimmedQuery = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmedQuery.isEmpty {
            let q = trimmedQuery.lowercased()
            result = result.filter { row in
                row.name.lowercased().contains(q) ||
                row.user.lowercased().contains(q) ||
                String(row.id).contains(q)
            }
        }

        // 3) Sorting
        result.sort { lhs, rhs in
            let cmp: ComparisonResult

            switch sortColumn {
            case .name:
                cmp = lhs.name.localizedCaseInsensitiveCompare(rhs.name)
            case .pid:
                cmp = ProcessListView.compare(lhs.id, rhs.id)
            case .user:
                cmp = lhs.user.localizedCaseInsensitiveCompare(rhs.user)
            case .cpu:
                cmp = ProcessListView.compare(lhs.cpuPercent, rhs.cpuPercent)
            case .memory:
                cmp = ProcessListView.compare(lhs.memoryMb, rhs.memoryMb)
            }

            if sortAscending {
                return cmp == .orderedAscending
            } else {
                return cmp == .orderedDescending
            }
        }

        // 4) Pinned rows float to top (preserving intra-group order).
        let pinned = result.filter { pinnedPids.contains($0.id) }
        let others = result.filter { !pinnedPids.contains($0.id) }

        return pinned + others
    }

    // MARK: - Body

    var body: some View {
        VStack(spacing: 8) {
            toolbar
            tableHeader

            if filteredAndSortedRows.isEmpty {
                emptyState
            } else {
                processTable
            }
        }
        .padding(8)
    }

    // MARK: - Toolbar

    private var toolbar: some View {
        HStack(spacing: 8) {
            // Search
            HStack(spacing: 6) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("Search by name, user, or PID", text: $searchText)
                    .textFieldStyle(.plain)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(NSColor.controlBackgroundColor))
            )

            // Toggles
            Toggle(isOn: $showSystemProcesses) {
                Text("Show system processes")
            }
            .toggleStyle(.switch)
            .font(.caption)

            Spacer()

            // Small count label
            Text("\(filteredAndSortedRows.count) processes")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 4)
    }

    // MARK: - Header Row

    private var tableHeader: some View {
        HStack(spacing: 0) {
            headerCell(.name)
            headerCell(.pid)
            headerCell(.user)
            headerCell(.cpu)
            headerCell(.memory)
        }
        .padding(.horizontal, 4)
        .padding(.vertical, 4)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(NSColor.controlBackgroundColor))
        )
    }

    private func headerCell(_ column: ProcessColumn) -> some View {
        Button {
            handleSortTap(on: column)
        } label: {
            HStack(spacing: 4) {
                Text(column.title)
                if sortColumn == column {
                    Image(systemName: sortAscending ? "arrow.up" : "arrow.down")
                        .font(.system(size: 10, weight: .bold))
                }
            }
            .font(.caption)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: column.alignment)
        }
        .buttonStyle(.plain)
        .frame(minWidth: column.minWidth)
    }

    private func handleSortTap(on column: ProcessColumn) {
        if column == sortColumn {
            // Toggle ascending/descending on same column
            sortAscending.toggle()
        } else {
            // Switch to new column; descending by default for numeric columns
            sortColumn = column
            switch column {
            case .cpu, .memory:
                sortAscending = false
            default:
                sortAscending = true
            }
        }
    }

    // MARK: - Table Body

    private var processTable: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 2) {
                ForEach(filteredAndSortedRows) { row in
                    ProcessRowView(
                        row: row,
                        pinned: pinnedPids.contains(row.id),
                        onTogglePinned: { togglePinned(row.id) }
                    )
                    .padding(.horizontal, 4)
                    .padding(.vertical, 2)
                }
            }
            .padding(.vertical, 4)
        }
        .background(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(NSColor.separatorColor), lineWidth: 0.5)
        )
    }

    private func togglePinned(_ pid: Int) {
        if pinnedPids.contains(pid) {
            pinnedPids.remove(pid)
        } else {
            pinnedPids.insert(pid)
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        RoundedRectangle(cornerRadius: 8)
            .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4, 4]))
            .foregroundStyle(Color.secondary.opacity(0.4))
            .frame(minHeight: 160)
            .overlay(
                VStack(spacing: 6) {
                    Text("No processes to display")
                        .font(.subheadline)
                    Text("Once backend wiring is in place, live process data will appear here.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding()
            )
    }

    // MARK: - Helpers

    private static func compare<T: Comparable>(_ lhs: T, _ rhs: T) -> ComparisonResult {
        if lhs == rhs { return .orderedSame }
        return lhs < rhs ? .orderedAscending : .orderedDescending
    }
}

// MARK: - Row View

private struct ProcessRowView: View {
    let row: ProcessRowModel
    let pinned: Bool
    let onTogglePinned: () -> Void

    private var cpuText: String {
        String(format: "%.1f", row.cpuPercent)
    }

    private var memText: String {
        String(format: "%.0f", row.memoryMb)
    }

    private var rowBackground: some View {
        Group {
            if pinned {
                RoundedRectangle(cornerRadius: 6)
                    .fill(Color.accentColor.opacity(0.08))
            } else if row.cpuPercent > 70 {
                RoundedRectangle(cornerRadius: 6)
                    .fill(Color.orange.opacity(0.06))
            } else {
                RoundedRectangle(cornerRadius: 6)
                    .fill(Color.clear)
            }
        }
    }

    var body: some View {
        ZStack {
            rowBackground

            HStack(spacing: 0) {
                // Name + pin / system badge
                HStack(spacing: 6) {
                    Button(action: onTogglePinned) {
                        Image(systemName: pinned ? "pin.fill" : "pin")
                            .font(.system(size: 10, weight: .medium))
                            .foregroundStyle(pinned ? .accentColor.opacity(0.9) : Color.secondary)
                    }
                    .buttonStyle(.plain)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(row.name)
                            .font(.system(size: 13, weight: .medium))
                            .lineLimit(1)
                        HStack(spacing: 6) {
                            Text("PID \(row.id)")
                            Text(row.user)
                        }
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    }

                    if row.isSystem {
                        Text("System")
                            .font(.caption2)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(
                                Capsule()
                                    .fill(Color.gray.opacity(0.16))
                            )
                    }

                    Spacer()
                }
                .frame(minWidth: ProcessColumn.name.minWidth, alignment: .leading)

                // PID column (compact, but we already show PID in subtitle – this is for strict tabular layout)
                Text("\(row.id)")
                    .font(.system(size: 12, weight: .regular, design: .monospaced))
                    .frame(minWidth: ProcessColumn.pid.minWidth,
                           maxWidth: .infinity,
                           alignment: .trailing)

                // User column
                Text(row.user)
                    .font(.system(size: 12))
                    .lineLimit(1)
                    .frame(minWidth: ProcessColumn.user.minWidth,
                           maxWidth: .infinity,
                           alignment: .leading)

                // CPU %
                Text(cpuText)
                    .font(.system(size: 12, weight: .semibold, design: .monospaced))
                    .foregroundStyle(row.cpuPercent > 70 ? Color.orange : Color.primary)
                    .frame(minWidth: ProcessColumn.cpu.minWidth,
                           maxWidth: .infinity,
                           alignment: .trailing)

                // Memory MB
                Text(memText)
                    .font(.system(size: 12, weight: .regular, design: .monospaced))
                    .frame(minWidth: ProcessColumn.memory.minWidth,
                           maxWidth: .infinity,
                           alignment: .trailing)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 4)
        }
    }
}

// MARK: - Preview

#Preview("Process List") {
    let mockRows: [ProcessRowModel] = [
        .init(id: 123,
              name: "Xcode",
              user: "root_shine",
              cpuPercent: 54.3,
              memoryMb: 1840,
              isSystem: false),
        .init(id: 456,
              name: "WindowServer",
              user: "_windowserver",
              cpuPercent: 8.2,
              memoryMb: 620,
              isSystem: true),
        .init(id: 789,
              name: "OverFlowBackend",
              user: "root_shine",
              cpuPercent: 72.9,
              memoryMb: 430,
              isSystem: false),
        .init(id: 1011,
              name: "kernel_task",
              user: "root",
              cpuPercent: 3.1,
              memoryMb: 256,
              isSystem: true)
    ]

    return ProcessListView(rows: mockRows)
        .frame(minWidth: 900, minHeight: 400)
        .padding()
}
