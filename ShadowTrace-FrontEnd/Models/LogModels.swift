//
//  LogModels.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

// MARK: - System Log

/// One line from `system_log.json` (newline-delimited JSON).
/// We keep both the raw line and, if possible, a parsed BackendEventEnvelope.
struct SystemLogEvent: Identifiable {
    let id = UUID()
    let lineNumber: Int
    let rawLine: String
    let envelope: BackendEventEnvelope?

    var timestamp: TimeInterval? {
        envelope?.timestamp
    }
}
// MARK: - File Access CSV

/// Parsed row from `file_access.csv`.
struct FileAccessEvent: Identifiable {
    let id = UUID()

    let timestamp: TimeInterval?
    let pid: Int
    let processName: String
    let path: String
    let action: String    // e.g. "read" / "write" / "delete"

    var date: Date? {
        guard let ts = timestamp else { return nil }
        return Date(timeIntervalSince1970: ts)
    }
}

// MARK: - Plugin Log

/// Parsed event from `plugin_log.json` (newline-delimited JSON).
struct PluginEvent: Identifiable, Decodable {
    let id = UUID()

    let timestamp: TimeInterval?
    let pluginId: String
    let pluginName: String?
    let level: String?        // "info", "warn", "error", ...
    let message: String

    enum CodingKeys: String, CodingKey {
        case timestamp
        case pluginId        = "plugin_id"
        case pluginName      = "plugin_name"
        case level
        case message
    }

    var date: Date? {
        guard let ts = timestamp else { return nil }
        return Date(timeIntervalSince1970: ts)
    }
}

// MARK: - Snapshot Files

/// Minimal description of a snapshot file in `logs/snapshots`.
struct SnapshotMetadata: Identifiable, Comparable {
    let id: UUID
    let filename: String
    let url: URL
    let createdAt: Date?

    static func < (lhs: SnapshotMetadata, rhs: SnapshotMetadata) -> Bool {
        switch (lhs.createdAt, rhs.createdAt) {
        case let (l?, r?): return l < r
        case (nil, _?):   return true
        case (_?, nil):   return false
        default:          return lhs.filename < rhs.filename
        }
    }
}

/// Parsed contents of a snapshot JSON file.
/// We keep it generic as a JSONValue tree so UI can render it however it wants.
struct SnapshotEvent {
    let id: UUID
    let raw: JSONValue
}

// MARK: - Streaming

/// Chunk produced when streaming a log file line-by-line.
struct StreamedLogChunk {
    let lineNumber: Int
    let rawLine: String
}
