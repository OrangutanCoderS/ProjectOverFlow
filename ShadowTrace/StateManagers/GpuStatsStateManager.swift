//
//  GpuStatsStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


import Foundation
import Combine

@MainActor
final class GpuStatsStateManager: ObservableObject {
    static let shared = GpuStatsStateManager()

    @Published private(set) var lastGpu: GpuStatsEvent?
    @Published private(set) var usageHistory: [GpuUsageSample] = []

    func update(_ evt: GpuStatsEvent) {
        lastGpu = evt

        // Push into history buffer (UI graphs already expect this pattern)
        let sample = GpuUsageSample(
            timestamp: Date().timeIntervalSince1970,
            usage: evt.usagePct
        )

        usageHistory.append(sample)

        // Prevent memory blow-up
        if usageHistory.count > 500 {
            usageHistory.removeFirst(200)
        }
    }
}