//
//  SensorHealth.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  SensorHealth.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

/// UI health classification for a sensor's *current* value.
/// Thresholding is done by whatever adapter feeds `SensorReading`.
enum SensorHealth: String, Codable {
    case normal
    case elevated
    case warning
    case critical
    case unknown

    var label: String {
        switch self {
        case .normal:   return "Normal"
        case .elevated: return "Elevated"
        case .warning:  return "Warning"
        case .critical: return "Critical"
        case .unknown:  return "Unknown"
        }
    }

    var color: Color {
        switch self {
        case .normal:   return .green
        case .elevated: return .yellow
        case .warning:  return .orange
        case .critical: return .red
        case .unknown:  return .gray
        }
    }
}