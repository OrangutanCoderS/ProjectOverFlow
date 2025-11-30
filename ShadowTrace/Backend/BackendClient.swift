//
//  BackendClient.swift
//  ShadowTrace
//
//  Backend/BackendClient.swift
//

import Foundation
import Combine
import Network

// MARK: - Connection Status

enum ConnectionStatus: Equatable {
    case disconnected
    case connecting
    case connected
    case error(String)
}

// MARK: - Transport Mode & Config

enum BackendTransportMode {
    case tcp(host: String, port: Int)
    case unixDomainSocket(path: String)
    case stdio
}

struct BackendConnectionConfig {
    let mode: BackendTransportMode

    static var `default`: BackendConnectionConfig {
        // Phase I demo → bundled Rust backend via stdio
        .init(mode: .stdio)
    }
}

// MARK: - Backend Command

enum SnapshotKind: String, Codable {
    case full
    case processes
    case files
    case network
}

struct TimeRange: Codable {
    let from: Date
    let to: Date
}

enum BackendCommand {
    case killProcess(pid: Int)
    case suspendProcess(pid: Int)
    case throttleProcess(pid: Int, cpuPercent: Double)
    case injectEnv(pid: Int, key: String, value: String)
    case fakeOutput(pid: Int, stdout: String?)

    case reloadPolicies
    case requestSnapshot(kind: SnapshotKind)
    case requestTimelineWindow(range: TimeRange)

    case enableNetworkRedirect(ruleId: String)
    case disableNetworkRedirect(ruleId: String)
}

// MARK: - Command Envelope

private struct BackendCommandEnvelope: Encodable {
    enum CodingKeys: String, CodingKey {
        case kind
        case command
        case payload
    }

    enum PayloadKeys: String, CodingKey {
        case pid
        case cpuPercent
        case key
        case value
        case stdout
        case snapshotKind
        case from
        case to
        case ruleId
    }

    let command: BackendCommand

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode("command", forKey: .kind)
        try container.encode(command.commandName, forKey: .command)

        var payload = container.nestedContainer(keyedBy: PayloadKeys.self, forKey: .payload)

        switch command {
        case .killProcess(let pid):
            try payload.encode(pid, forKey: .pid)

        case .suspendProcess(let pid):
            try payload.encode(pid, forKey: .pid)

        case .throttleProcess(let pid, let cpuPercent):
            try payload.encode(pid, forKey: .pid)
            try payload.encode(cpuPercent, forKey: .cpuPercent)

        case .injectEnv(let pid, let key, let value):
            try payload.encode(pid, forKey: .pid)
            try payload.encode(key, forKey: .key)
            try payload.encode(value, forKey: .value)

        case .fakeOutput(let pid, let stdout):
            try payload.encode(pid, forKey: .pid)
            try payload.encodeIfPresent(stdout, forKey: .stdout)

        case .reloadPolicies:
            break

        case .requestSnapshot(let kind):
            try payload.encode(kind.rawValue, forKey: .snapshotKind)

        case .requestTimelineWindow(let range):
            try payload.encode(range.from, forKey: .from)
            try payload.encode(range.to, forKey: .to)

        case .enableNetworkRedirect(let ruleId),
             .disableNetworkRedirect(let ruleId):
            try payload.encode(ruleId, forKey: .ruleId)
        }
    }
}

private extension BackendCommand {
    var commandName: String {
        switch self {
        case .killProcess: return "kill_process"
        case .suspendProcess: return "suspend_process"
        case .throttleProcess: return "throttle_process"
        case .injectEnv: return "inject_env"
        case .fakeOutput: return "fake_output"
        case .reloadPolicies: return "reload_policies"
        case .requestSnapshot: return "request_snapshot"
        case .requestTimelineWindow: return "request_timeline_window"
        case .enableNetworkRedirect: return "enable_network_redirect"
        case .disableNetworkRedirect: return "disable_network_redirect"
        }
    }
}

// MARK: - BackendClient

@MainActor
final class BackendClient: ObservableObject {

    static let shared = BackendClient(config: .default)

    @Published private(set) var connectionStatus: ConnectionStatus = .disconnected

    private let config: BackendConnectionConfig
    private var connection: BackendConnection?
    private var listenTask: Task<Void, Never>?

    private let jsonDecoder: JSONDecoder
    private let jsonEncoder: JSONEncoder

    private var incomingBuffer = Data()

    // MARK: - Init

    init(config: BackendConnectionConfig) {
        self.config = config

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .secondsSince1970
        self.jsonDecoder = decoder

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .secondsSince1970
        encoder.outputFormatting = [.sortedKeys]
        self.jsonEncoder = encoder
    }

    deinit {
        Task { @MainActor [weak self] in
            self?.stop()
        }
    }

    // MARK: - Lifecycle

    func start() {
        guard listenTask == nil else { return }

        connectionStatus = .connecting

        let backendConnection: BackendConnection

        switch config.mode {

        case .tcp(let host, let port):
            backendConnection = TCPBackendConnection(host: host, port: port)

        case .unixDomainSocket(let path):
            // Unimplemented → fallback
            backendConnection = TCPBackendConnection(host: "127.0.0.1", port: 7878)
            print("[BackendClient] unix socket \(path) not implemented, fallback to TCP")

        case .stdio:
            // NEW: launch bundled Rust `system` binary
            backendConnection = ProcessBackendConnection(binaryName: "system")
        }

        self.connection = backendConnection

        backendConnection.startReceiving { [weak self] result in
            guard let self else { return }

            Task { @MainActor [weak self] in
                guard let self else { return }
                await self.handleReceive(result: result)
            }
        }

        listenTask = Task { [weak self] in
            guard let self else { return }
            await self.markConnected()
        }
    }

    func stop() {
        listenTask?.cancel()
        listenTask = nil
        connection?.close()
        connection = nil
        connectionStatus = .disconnected
    }

    func restart() {
        stop()
        start()
    }

    // MARK: - Public Send API

    func send(_ command: BackendCommand) {
        guard let connection else {
            print("[BackendClient] send() with no active connection")
            return
        }

        do {
            let envelope = BackendCommandEnvelope(command: command)
            let data = try jsonEncoder.encode(envelope)

            var framed = data
            framed.append(0x0A) // newline

            try connection.send(framed)
        } catch {
            print("[BackendClient] failed to send: \(error)")
        }
    }

    // MARK: - Receive Handler

    private func handleReceive(result: Result<Data, Error>) async {
        switch result {
        case .success(let chunk):
            await handleIncomingChunk(chunk)

        case .failure(let error):
            await handleConnectionError(error)
        }
    }

    private func handleIncomingChunk(_ data: Data) async {
        incomingBuffer.append(data)

        while let newlineRange = incomingBuffer.firstRange(of: Data([0x0A])) {
            let frame = incomingBuffer.subdata(in: 0..<newlineRange.lowerBound)
            incomingBuffer.removeSubrange(0...newlineRange.lowerBound)

            guard !frame.isEmpty else { continue }
            await handleFrame(frame)
        }
    }

    private func handleFrame(_ data: Data) async {
        do {
            let envelope = try jsonDecoder.decode(
                BackendEventEnvelope.self,
                from: data
            )

            let event = try EventDecoder.shared.decodeGenericEnvelope(envelope: envelope)
            BackendStateManager.shared.handle(event: event)

        } catch {
            logParseError(error, raw: data)
        }
    }

    private func handleConnectionError(_ error: Error) async {
        print("[BackendClient] connection error: \(error)")
        connectionStatus = .error(error.localizedDescription)
        stop()

        try? await Task.sleep(nanoseconds: 1_000_000_000)
        start()
    }

    private func markConnected() async {
        if case .connecting = connectionStatus {
            connectionStatus = .connected
        }
    }

    private func logParseError(_ error: Error, raw: Data) {
        let rawString = String(data: raw, encoding: .utf8) ?? "<non-UTF8>"
        print("[BackendClient] Failed to decode event: \(error)\nRaw: \(rawString)")
    }
}
