//
//  SensorCategory.swift
//  ShadowTrace
//

import Foundation

/// High-level bucket for hardware sensors (for grouping in UI).
enum SensorCategory: String, CaseIterable, Codable, Identifiable {
    case cpu
    case gpu
    case thermal
    case battery
    case fan
    case power
    case other

    var id: String { rawValue }

    var title: String {
        switch self {
        case .cpu:     return "CPU"
        case .gpu:     return "GPU"
        case .thermal: return "Thermal"
        case .battery: return "Battery"
        case .fan:     return "Fans"
        case .power:   return "Power"
        case .other:   return "Other"
        }
    }

    /// Sort order used in UI.
    var sortOrder: Int {
        switch self {
        case .cpu:     return 0
        case .gpu:     return 1
        case .thermal: return 2
        case .battery: return 3
        case .fan:     return 4
        case .power:   return 5
        case .other:   return 6
        }
    }

    /// Fallback mapping when backend sends a raw string.
    static func from(raw: String) -> SensorCategory {
        let lower = raw.lowercased()
        if lower.contains("cpu")      { return .cpu }
        if lower.contains("gpu")      { return .gpu }
        if lower.contains("temp") ||
           lower.contains("therm")    { return .thermal }
        if lower.contains("batt")     { return .battery }
        if lower.contains("fan")      { return .fan }
        if lower.contains("power") ||
           lower.contains("amp") ||
           lower.contains("volt")     { return .power }
        return .other
    }
}
