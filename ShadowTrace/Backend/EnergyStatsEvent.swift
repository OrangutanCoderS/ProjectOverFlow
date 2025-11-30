//
//  EnergyStatsEvent.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Thermal impact classification for a single process.
/// Backend can send these as lowercase strings: "low", "medium", "high", "critical".
enum ThermalImpact: String, Codable {
    case low
    case medium
    case high
    case critical
}

/// Per-process energy stats snapshot emitted by the backend.
///
/// Expected semantics (you can tune them on the Rust side later):
/// - energyScore: 0–100 normalized "badness" or consumption index
/// - cpuPercent / gpuPercent: contribution of CPU vs GPU to that energyScore
/// - wakeupsPerSec: approximate wakeups / second
/// - estimatedMilliwatts: backend-side approximation, if available
struct EnergyStatsEvent: Decodable {
    let pid: Int
    let processName: String?

    let energyScore: Double          // 0–100
    let cpuPercent: Double           // 0–100
    let gpuPercent: Double           // 0–100

    let thermal: ThermalImpact       // low/medium/high/critical

    let wakeupsPerSec: Int?
    let estimatedMilliwatts: Double?
}
