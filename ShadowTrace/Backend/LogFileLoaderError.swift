//
//  LogFileLoaderError.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

/// Errors that can occur while reading / decoding backend log files.
enum LogFileLoaderError: LocalizedError {
    case logsRootNotFound
    case fileNotFound(path: String)
    case unreadableFile(path: String, underlying: Error)
    case decodeFailed(path: String, lineNumber: Int?, underlying: Error)
    case invalidFormat(path: String, lineNumber: Int?)
    case directoryReadFailed(path: String, underlying: Error)

    var errorDescription: String? {
        switch self {
        case .logsRootNotFound:
            return "Could not determine OverFlow logs directory."

        case .fileNotFound(let path):
            return "Log file not found at path: \(path)"

        case .unreadableFile(let path, let underlying):
            return "Unable to read log file at path \(path): \(underlying.localizedDescription)"

        case .decodeFailed(let path, let lineNumber, let underlying):
            if let line = lineNumber {
                return "Failed to decode log entry in \(path) at line \(line): \(underlying.localizedDescription)"
            } else {
                return "Failed to decode log file \(path): \(underlying.localizedDescription)"
            }

        case .invalidFormat(let path, let lineNumber):
            if let line = lineNumber {
                return "Invalid log format in \(path) at line \(line)."
            } else {
                return "Invalid log format in \(path)."
            }

        case .directoryReadFailed(let path, let underlying):
            return "Failed to read directory \(path): \(underlying.localizedDescription)"
        }
    }
}
