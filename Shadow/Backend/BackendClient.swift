//  BackendClient.swift
//  ShadowTrace

import Foundation
import Combine

enum ConnectionStatus: Equatable {
    case disconnected
    case connecting
    case connected
    case error(String)
}

final class BackendClient {
    static let shared = BackendClient()

    // Public stream of decoded backend events
    var events: AnyPublisher<BackendEvent, Never> {
        eventSubject.eraseToAnyPublisher()
    }

    @Published private(set) var status: ConnectionStatus = .disconnected

    // MARK: - Private

    private let eventSubject = PassthroughSubject<BackendEvent, Never>()
    private let statusSubject = PassthroughSubject<ConnectionStatus, Never>()

    private let queue = DispatchQueue(label: "Overflow.BackendClient")
    private var process: Process?
    private let stdoutPipe = Pipe()
    private var buffer = Data()
    private var cancellables = Set<AnyCancellable>()

    private init() {
        // If you want external observers for status, expose statusSubject too.
        statusSubject
            .receive(on: DispatchQueue.main)
            .assign(to: &self.$status)
    }

    // MARK: - Public API

    func startIfNeeded() {
        guard process == nil else { return }
        launchBackend()
    }

    func stop() {
        queue.sync {
            self.stdoutPipe.fileHandleForReading.readabilityHandler = nil
            self.process?.terminate()
            self.process = nil
            self.buffer.removeAll()
            self.statusSubject.send(.disconnected)
        }
    }

    // MARK: - Launch & Stream

    private func launchBackend() {
        queue.async { [weak self] in
            guard let self = self else { return }

            if self.process != nil {
                return
            }

            let process = Process()
            process.standardOutput = self.stdoutPipe
            process.standardError = nil

            // TODO: set this to your actual embedded binary name/path.
            // 1. Try auxiliary executable in /Contents/MacOS/
            if let url = Bundle.main.url(forAuxiliaryExecutable: "system") {
                process.executableURL = url
            }
            // 2. Fallback: development binary inside target/debug
            else if FileManager.default.fileExists(atPath: "\(FileManager.default.currentDirectoryPath)/target/debug/system") {
                process.executableURL = URL(fileURLWithPath: "\(FileManager.default.currentDirectoryPath)/target/debug/system")
            }
            // 3. Fallback absolute path (optional)
            else {
                process.executableURL = URL(fileURLWithPath: "/Users/root1/Desktop/OverFlow/target/debug/system")
            }

            // If your binary needs CLI args, set them here:
            // process.arguments = ["--mode", "jsonl"]

            do {
                self.statusSubject.send(.connecting)

                self.stdoutPipe.fileHandleForReading.readabilityHandler = { [weak self] handle in
                    self?.handleReadableData(handle: handle)
                }

                try process.run()
                self.process = process
                self.statusSubject.send(.connected)

                process.terminationHandler = { [weak self] _ in
                    guard let self = self else { return }
                    self.stdoutPipe.fileHandleForReading.readabilityHandler = nil
                    self.queue.async {
                        self.process = nil
                        self.statusSubject.send(.disconnected)
                    }
                }
            } catch {
                self.statusSubject.send(.error("Failed to launch backend: \(error.localizedDescription)"))
            }
        }
    }

    // MARK: - Stream Parsing

    private func handleReadableData(handle: FileHandle) {
        let chunk = handle.availableData
        guard !chunk.isEmpty else {
            // EOF
            stop()
            return
        }

        buffer.append(chunk)

        while true {
            if let range = buffer.firstRange(of: Data([0x0A])) { // newline
                let lineData = buffer.subdata(in: 0..<range.lowerBound)
                buffer.removeSubrange(0...range.lowerBound)

                if !lineData.isEmpty {
                    decodeLine(lineData)
                }
            } else {
                break
            }
        }
    }

    private func decodeLine(_ data: Data) {
        do {
            let decoder = JSONDecoder()
            let envelope = try decoder.decode(BackendEnvelope.self, from: data)
            eventSubject.send(envelope.event)
        } catch {
            // For debugging, you can print the line:
            // let s = String(data: data, encoding: .utf8) ?? "<invalid utf8>"
            // print("Decode error: \(error), line=\(s)")
        }
    }
}
