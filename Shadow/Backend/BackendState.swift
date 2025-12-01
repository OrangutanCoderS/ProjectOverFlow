//  BackendState.swift
//  ShadowTrace

import Foundation
import Combine

final class BackendState: ObservableObject {

    // MARK: - Published metrics

    @Published var globalCpu: Double = 0
    @Published var perCoreCpu: [Double] = []

    @Published var memTotalMB: Double = 0
    @Published var memUsedMB: Double = 0

    @Published var gpuName: String = ""
    @Published var gpuUsage: Double = 0
    @Published var gpuTemperature: Double = 0

    @Published var batteryPercentage: Double = 0
    @Published var batteryCharging: Bool = false
    @Published var batteryTimeRemainingMin: Double? = nil

    @Published var connectionStatus: ConnectionStatus = .disconnected

    // derived
    var memUsagePercent: Double {
        guard memTotalMB > 0 else { return 0 }
        return (memUsedMB / memTotalMB) * 100.0
    }

    // MARK: - Private

    private let client: BackendClient
    private var cancellables = Set<AnyCancellable>()

    init(client: BackendClient = .shared) {
        self.client = client
        bind()
    }

    private func bind() {
        client.events
            .receive(on: DispatchQueue.main)
            .sink { [weak self] event in
                guard let self else { return }
                switch event {
                case .cpu(let e):
                    self.globalCpu = e.global_usage
                    self.perCoreCpu = e.per_core

                case .systemStats(let e):
                    self.memTotalMB = e.mem_total_mb
                    self.memUsedMB = e.mem_used_mb

                case .gpu(let e):
                    self.gpuName = e.name
                    self.gpuUsage = e.usage_pct
                    self.gpuTemperature = e.temperature_c

                case .battery(let e):
                    self.batteryPercentage = e.percentage
                    self.batteryCharging = e.charging
                    self.batteryTimeRemainingMin = e.time_remaining_min
                }
            }
            .store(in: &cancellables)

        client.$status
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                self?.connectionStatus = status
            }
            .store(in: &cancellables)
    }

    // Exposed control for the UI
    func start() {
        client.startIfNeeded()
    }

    func stop() {
        client.stop()
    }
}
