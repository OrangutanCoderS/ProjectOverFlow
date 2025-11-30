//
//  SensorUpdateEvent.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  SensorUpdateEvent.swift
//  ShadowTrace
//
//  Created by root_shine on 11/30/25.
//


//
//  SensorUpdateEvent.swift
//  ShadowTrace
//

import Foundation

/// Backend → frontend envelope for a single sensor update.
/// Decoded from JSON inside BackendEventEnvelope / EventDecoder.
struct SensorUpdateEvent: Decodable {
    let key: String           // "BAT0-Level", "BAT0-Temp", etc.
    let label: String         // "Battery Level"
    let categoryRaw: String   // "battery", "thermal", "fan", etc.
    let value: Double?        // Some sensors may still send nil
    let unit: String?
    let sortOrder: Int?

    enum CodingKeys: String, CodingKey {
        case key
        case label
        case categoryRaw = "category"
        case value
        case unit
        case sortOrder
    }

    var category: SensorCategory {
        SensorCategory(rawValue: categoryRaw.lowercased()) ??
        SensorCategory.from(raw: categoryRaw)
    }
}

