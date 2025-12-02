//
//  SensorReading.swift
//  ShadowTrace
//

import Foundation

/// View-model for a single hardware sensor value.
struct SensorReading: Identifiable, Equatable {

    /// Stable id – we reuse the key.
    var id: String { key }

    let key: String          // backend/SMC key, e.g. "TC0P"
    let label: String        // human label, e.g. "CPU Proximity"
    var value: Double?        // current numeric value
    var unit: String         // "°C", "RPM", "W", etc.
    let category: SensorCategory

    /// Used for ordering within a category.
    var sortOrder: Int

    var lastUpdated: Date
}
