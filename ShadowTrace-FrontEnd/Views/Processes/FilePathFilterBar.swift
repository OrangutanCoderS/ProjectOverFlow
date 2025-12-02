//
//  FilePathFilterBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  FilePathFilterBar.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// Reusable filter bar for file-path filtering and event-type filtering.
/// Used in:
/// - FileActivityView
/// - Future ProcessFileTab
/// - Global File Monitor (Phase-next)
///
struct FilePathFilterBar: View {

    // MARK: - Bindings from parent

    @Binding var searchQuery: String
    @Binding var selectedAction: FileAction?     // nil = ALL

    var onSearchChanged: (String) -> Void
    var onActionChanged: (FileAction?) -> Void

    // MARK: - Focus for search field
    @FocusState private var isSearchFocused: Bool

    // MARK: - Body
    var body: some View {
        VStack(spacing: 10) {

            // SEARCH BAR
            HStack(spacing: 8) {
                HStack(spacing: 6) {
                    Image(systemName: "magnifyingglass")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)

                    TextField("Search files…", text: $searchQuery)
                        .textFieldStyle(.plain)
                        .font(.subheadline)
                        .focused($isSearchFocused)
                        .onChange(of: searchQuery) { new in
                            onSearchChanged(new)
                        }

                    if !searchQuery.isEmpty {
                        Button {
                            searchQuery = ""
                            onSearchChanged("")
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundStyle(.secondary)
                        }
                        .buttonStyle(.borderless)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .fill(Color(NSColor.controlBackgroundColor))
                )
            }

            // ACTION PILLS
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    actionPill(label: "All", action: nil, color: .gray)

                    actionPill(label: "Created", action: .created, color: .green)
                    actionPill(label: "Modified", action: .modified, color: .yellow)
                    actionPill(label: "Deleted", action: .deleted, color: .red)
                    actionPill(label: "Read", action: .read, color: .blue)
                    actionPill(label: "Renamed", action: .renamed, color: .orange)
                    actionPill(label: "Metadata", action: .metadata, color: .purple)
                }
                .padding(.horizontal, 2)
            }
        }
        .padding(.vertical, 4)
    }

    // MARK: - Pill Builder

    @ViewBuilder
    private func actionPill(label: String, action: FileAction?, color: Color) -> some View {

        let isSelected = (selectedAction == action)

        Button {
            selectedAction = action
            onActionChanged(action)
        } label: {
            HStack(spacing: 6) {
                Circle()
                    .fill(color.opacity(isSelected ? 1.0 : 0.4))
                    .frame(width: 6, height: 6)

                Text(label)
                    .font(.caption)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(
                Capsule()
                    .fill(isSelected
                          ? color.opacity(0.18)
                          : Color(NSColor.controlBackgroundColor))
            )
            .overlay(
                Capsule()
                    .strokeBorder(isSelected ? color.opacity(0.8) : .clear, lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Preview

#Preview("Filter Bar") {
    @State var query = ""
    @State var action: FileAction? = nil

    return FilePathFilterBar(
        searchQuery: $query,
        selectedAction: $action,
        onSearchChanged: { _ in },
        onActionChanged: { _ in }
    )
    .frame(width: 620)
    .padding()
    .environment(\.colorScheme, .dark)
}