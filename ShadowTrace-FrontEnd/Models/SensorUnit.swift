//
//  SensorUnit.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  SensorUnit.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Normalized unit for a sensor reading.
/// Backend can still send raw unit strings – adapter maps them into this.
enum SensorUnit: String, Codable {
    case celsius
    case fahrenheit
    case percent
    case watts
    case volts
    case amps
    case rpm
    case hertz
    case bytesPerSecond
    case none
    case unknown

    var shortLabel: String {
        switch self {
        case .celsius:       return "°C"
        case .fahrenheit:    return "°F"
        case .percent:       return "%"
        case .watts:         return "W"
        case .volts:         return "V"
        case .amps:          return "A"
        case .rpm:           return "RPM"
        case .hertz:         return "Hz"
        case .bytesPerSecond:return "B/s"
        case .none:          return ""
        case .unknown:       return "?"
        }
    }
}