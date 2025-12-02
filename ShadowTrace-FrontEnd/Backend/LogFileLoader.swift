//
//  LogFileLoader.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

final class LogFileLoader {

    static let shared = LogFileLoader()

    private let fileManager = FileManager.default
    private let jsonDecoder: JSONDecoder
    private let logsRoot: URL

    init(logsRoot: URL? = nil) {
        self.jsonDecoder = {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .secondsSince1970
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            return decoder
        }()

        if let explicit = logsRoot {
            self.logsRoot = explicit
        } else if let appSupport = try? fileManager.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: false
        ) {
            self.logsRoot = appSupport
                .appendingPathComponent("OverFlow", isDirectory: true)
                .appendingPathComponent("logs", isDirectory: true)
        } else {
            let bundleRoot = Bundle.main.bundleURL.deletingLastPathComponent()
            self.logsRoot = bundleRoot.appendingPathComponent("logs", isDirectory: true)
        }
    }

    // MARK: - Log File Paths

    private func systemLogURL() -> URL {
        logsRoot.appendingPathComponent("system_log.json")
    }

    private func fileAccessCSVURL() -> URL {
        logsRoot.appendingPathComponent("file_access.csv")
    }

    private func pluginLogURL() -> URL {
        logsRoot.appendingPathComponent("plugin_log.json")
    }

    private func snapshotsDirectoryURL() -> URL {
        logsRoot.appendingPathComponent("snapshots", isDirectory: true)
    }

    // MARK: - System Log Loading

    func loadSystemLog() async throws -> [SystemLogEvent] {
        let url = systemLogURL()
        guard fileManager.fileExists(atPath: url.path) else {
            throw LogFileLoaderError.fileNotFound(path: url.path)
        }

        let data = try Data(contentsOf: url)

        guard let text = String(data: data, encoding: .utf8) else {
            throw LogFileLoaderError.invalidFormat(path: url.path, lineNumber: nil)
        }

        var events: [SystemLogEvent] = []
        events.reserveCapacity(512)

        let lines = text.split(whereSeparator: \.isNewline)

        for (index, substring) in lines.enumerated() {
            let line = String(substring)
            let lineNumber = index + 1

            let envelope: BackendEventEnvelope?
            if let encoded = line.data(using: .utf8) {
                envelope = try? jsonDecoder.decode(BackendEventEnvelope.self, from: encoded)
            } else {
                envelope = nil
            }

            events.append(SystemLogEvent(
                lineNumber: lineNumber,
                rawLine: line,
                envelope: envelope
            ))
        }

        return events
    }

    func tailSystemLog(lines count: Int) async throws -> [SystemLogEvent] {
        let all = try await loadSystemLog()
        guard count < all.count else { return all }
        return Array(all.suffix(count))
    }

    // MARK: - Async Stream (fixed for macOS 15+)

    func streamSystemLog() -> AsyncStream<SystemLogEvent> {
        let url = systemLogURL()

        return AsyncStream { continuation in
            Task.detached { [jsonDecoder] in
                defer { continuation.finish() }

                guard FileManager.default.fileExists(atPath: url.path) else { return }

                do {
                    let handle = try FileHandle(forReadingFrom: url)
                    var buffer = Data()
                    var lineNumber = 0

                    for try await byte in handle.bytes {
                        if byte == UInt8(ascii: "\n") {
                            lineNumber += 1

                            let rawChunk = buffer
                            buffer.removeAll(keepingCapacity: true)

                            guard let line = String(data: rawChunk, encoding: .utf8) else { continue }

                            let envelope: BackendEventEnvelope?
                            if let encoded = line.data(using: .utf8) {
                                envelope = try? jsonDecoder.decode(BackendEventEnvelope.self, from: encoded)
                            } else {
                                envelope = nil
                            }

                            continuation.yield(SystemLogEvent(
                                lineNumber: lineNumber,
                                rawLine: line,
                                envelope: envelope
                            ))
                        } else {
                            buffer.append(byte)
                        }
                    }

                    // Handle last line (if file doesn't end with newline)
                    if !buffer.isEmpty {
                        lineNumber += 1
                        let line = String(data: buffer, encoding: .utf8) ?? ""
                        let envelope = try? jsonDecoder.decode(BackendEventEnvelope.self, from: Data(buffer))

                        continuation.yield(SystemLogEvent(
                            lineNumber: lineNumber,
                            rawLine: line,
                            envelope: envelope
                        ))
                    }

                } catch {
                    print("[LogFileLoader] streamSystemLog error: \(error)")
                }
            }
        }
    }

    // MARK: - CSV Parsing

    func loadFileAccessCSV() async throws -> [FileAccessEvent] {
        let url = fileAccessCSVURL()
        guard fileManager.fileExists(atPath: url.path) else {
            throw LogFileLoaderError.fileNotFound(path: url.path)
        }

        let data = try Data(contentsOf: url)

        guard let text = String(data: data, encoding: .utf8) else {
            throw LogFileLoaderError.invalidFormat(path: url.path, lineNumber: nil)
        }

        var events: [FileAccessEvent] = []

        let lines = text.split(whereSeparator: \.isNewline)
        guard !lines.isEmpty else { return [] }

        for substring in lines.dropFirst() {
            let cols = parseCSVLine(String(substring))
            if cols.count < 5 { continue }

            let event = FileAccessEvent(
                timestamp: Double(cols[0]),
                pid: Int(cols[1]) ?? -1,
                processName: cols[2],
                path: cols[3],
                action: cols[4]
            )
            events.append(event)
        }

        return events
    }

    private func parseCSVLine(_ line: String) -> [String] {
        line.split(separator: ",")
            .map { $0.trimmingCharacters(in: .whitespaces) }
    }

    // MARK: - Plugin Log

    func loadPluginLog() async throws -> [PluginEvent] {
        let url = pluginLogURL()

        guard fileManager.fileExists(atPath: url.path) else {
            throw LogFileLoaderError.fileNotFound(path: url.path)
        }

        let data = try Data(contentsOf: url)

        guard let text = String(data: data, encoding: .utf8) else {
            throw LogFileLoaderError.invalidFormat(path: url.path, lineNumber: nil)
        }

        var events: [PluginEvent] = []

        let lines = text.split(whereSeparator: \.isNewline)

        for substring in lines {
            let raw = String(substring)
            if let data = raw.data(using: .utf8),
               let event = try? jsonDecoder.decode(PluginEvent.self, from: data) {
                events.append(event)
            }
        }

        return events
    }
}
