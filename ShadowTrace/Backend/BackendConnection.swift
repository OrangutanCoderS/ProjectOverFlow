//  BackendConnection.swift
//  ShadowTrace
//
//  Backend/BackendConnection.swift

import Foundation
import Network

// MARK: - Protocol

protocol BackendConnection {
    /// Start receiving bytes from the backend.
    /// The callback will be invoked on a background queue.
    func startReceiving(onEvent: @escaping (Result<Data, Error>) -> Void)

    /// Send a raw frame to the backend.
    func send(_ data: Data) throws

    /// Close the underlying connection.
    func close()
}

// MARK: - Common Error

enum BackendConnectionError: Error {
    case connectionClosed
    case binaryNotFound(String)
}

// MARK: - TCP Implementation (kept for future remote mode)

final class TCPBackendConnection: BackendConnection {

    private let host: NWEndpoint.Host
    private let port: NWEndpoint.Port
    private let queue = DispatchQueue(label: "Overflow.Backend.TCPConnection")

    private var connection: NWConnection?

    init(host: String, port: Int) {
        self.host = NWEndpoint.Host(host)
        self.port = NWEndpoint.Port(rawValue: UInt16(port))!
    }

    func startReceiving(onEvent: @escaping (Result<Data, Error>) -> Void) {
        let conn = NWConnection(host: host, port: port, using: .tcp)
        self.connection = conn

        conn.stateUpdateHandler = { state in
            switch state {
            case .ready:
                self.receiveNext(onEvent: onEvent)
            case .failed(let error):
                onEvent(.failure(error))
            case .cancelled:
                onEvent(.failure(BackendConnectionError.connectionClosed))
            default:
                break
            }
        }

        conn.start(queue: queue)
    }

    private func receiveNext(onEvent: @escaping (Result<Data, Error>) -> Void) {
        connection?.receive(minimumIncompleteLength: 1,
                            maximumLength: 64 * 1024) { data, _, isComplete, error in
            if let error = error {
                onEvent(.failure(error))
                return
            }

            if let data = data, !data.isEmpty {
                onEvent(.success(data))
            }

            if isComplete {
                onEvent(.failure(BackendConnectionError.connectionClosed))
                return
            }

            self.receiveNext(onEvent: onEvent)
        }
    }

    func send(_ data: Data) throws {
        guard let connection else {
            throw BackendConnectionError.connectionClosed
        }

        connection.send(content: data, completion: .contentProcessed { error in
            if let error = error {
                print("[TCPBackendConnection] send error: \(error)")
            }
        })
    }

    func close() {
        connection?.cancel()
        connection = nil
    }
}

// MARK: - Process / stdio Implementation (bundled Rust binary)

final class ProcessBackendConnection: BackendConnection {

    private let binaryName: String
    private let queue = DispatchQueue(label: "Overflow.Backend.ProcessConnection")

    private var process: Process?
    private var stdoutPipe: Pipe?
    private var stdinPipe: Pipe?

    init(binaryName: String) {
        self.binaryName = binaryName
    }

    func startReceiving(onEvent: @escaping (Result<Data, Error>) -> Void) {
        // Resolve path to bundled executable:
        // we copied `system` into the app’s Executables (Contents/MacOS).
        let bundleURL = Bundle.main.bundleURL
        let execDir = bundleURL.appendingPathComponent("Contents/MacOS", isDirectory: true)
        let binaryURL = execDir.appendingPathComponent(binaryName, isDirectory: false)

        guard FileManager.default.isExecutableFile(atPath: binaryURL.path) else {
            onEvent(.failure(BackendConnectionError.binaryNotFound(binaryURL.path)))
            return
        }

        let proc = Process()
        proc.executableURL = binaryURL
        proc.arguments = [] // our Rust binary needs no args for now

        let outPipe = Pipe()
        let inPipe = Pipe()
        let errPipe = Pipe() // we can log stderr later if needed

        proc.standardOutput = outPipe
        proc.standardError = errPipe
        proc.standardInput  = inPipe

        self.process = proc
        self.stdoutPipe = outPipe
        self.stdinPipe = inPipe

        proc.terminationHandler = { _ in
            onEvent(.failure(BackendConnectionError.connectionClosed))
        }

        do {
            try proc.run()
        } catch {
            onEvent(.failure(error))
            return
        }

        // Continuous blocking read on a background queue.
        let handle = outPipe.fileHandleForReading
        queue.async { [weak self] in
            guard self != nil else { return }

            while true {
                autoreleasepool {
                    let chunk = handle.readData(ofLength: 64 * 1024)
                    if chunk.isEmpty {
                        // EOF
                        onEvent(.failure(BackendConnectionError.connectionClosed))
                        return
                    } else {
                        onEvent(.success(chunk))
                    }
                }
            }
        }
    }

    func send(_ data: Data) throws {
        // Currently the Rust `system` binary ignores stdin; we still wire it for future commands.
        guard let pipe = stdinPipe else {
            throw BackendConnectionError.connectionClosed
        }
        pipe.fileHandleForWriting.write(data)
    }

    func close() {
        stdoutPipe?.fileHandleForReading.closeFile()
        stdinPipe?.fileHandleForWriting.closeFile()
        process?.terminate()
        process = nil
        stdoutPipe = nil
        stdinPipe = nil
    }
}
