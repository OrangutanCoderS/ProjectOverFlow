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
            levelPercent: evt.percentage,     // rename
            isCharging: evt.charging,
            onACPower: false,                 // depends on evt, you don’t have this yet
            healthPercent: nil,
            cycleCount: nil,
            temperatureC: nil,
            powerWatts: nil
        )
        }
    }

