//
//  FileAction.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Unified file operation class used across:
/// - FileActivityView (operation pills)
/// - FilePathFilterBar
/// - FileEventInspector
///
/// IMPORTANT:
/// This enum **must** stay in sync with:
/// FileOpFilter (in FileActivityView)
/// FileEvent.opKind (mapping backend → UI)
///
enum FileAction: String, CaseIterable, Identifiable {

    // MARK: - Supported high-level actions
    case read
    case write
    case created
    case deleted
    case renamed
    case modified        // patched
    case metadata        // fallback / extended op category

    // MARK: - Identifiable
    var id: String { rawValue }

    // MARK: - UI Labels
    var label: String {
        switch self {
        case .read:     return "Read"
        case .write:    return "Write"
        case .created:  return "Created"
        case .deleted:  return "Deleted"
        case .renamed:  return "Renamed"
        case .modified: return "Modified"
        case .metadata: return "Metadata"
        }
    }
}
