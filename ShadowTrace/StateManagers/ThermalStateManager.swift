//
//  ThermalStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


import Foundation
import Combine

@MainActor
final class ThermalStateManager: ObservableObject {
    static let shared = ThermalStateManager()

    @Published private(set) var lastThermal: ThermalStatsEvent?
    @Published private(set) var samples: [ThermalSample] = []

    func update(_ evt: ThermalStatsEvent) {
        lastThermal = evt

        let sample = ThermalSample(
            timestamp: Date().timeIntervalSince1970,
            cpuTempC: evt.cpuTempC ?? 0,
            gpuTempC: evt.gpuTempC ?? 0,
            skinTempC: evt.skinTempC ?? 0
        )

        samples.append(sample)

        if samples.count > 500 {
            samples.removeFirst(200)
        }
    }
}