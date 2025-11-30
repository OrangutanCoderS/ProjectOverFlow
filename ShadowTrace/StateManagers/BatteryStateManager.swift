//
//  BatteryStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


import Foundation
import Combine

@MainActor
final class BatteryStateManager: ObservableObject {
    static let shared = BatteryStateManager()

    @Published private(set) var lastBattery: BatteryStatsEvent?
    @Published private(set) var history: [BatteryHistoryPoint] = []

    func update(_ evt: BatteryStatsEvent) {
        lastBattery = evt

        let point = BatteryHistoryPoint(
            timestamp: Date().timeIntervalSince1970,
            percentage: evt.percentage,
            isCharging: evt.charging
        )

        history.append(point)

        if history.count > 500 {
            history.removeFirst(200)
        }
    }
}