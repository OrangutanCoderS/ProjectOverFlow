//
//  CpuLoadLevel.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  CpuLoadLevel.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import SwiftUI

enum CpuLoadLevel {
    case idle
    case normal
    case busy
    case hot
    case critical

    init(percent: Double) {
        let value = max(0, percent)
        switch value {
        case 0..<1:
            self = .idle
        case 1..<20:
            self = .normal
        case 20..<60:
            self = .busy
        case 60..<90:
            self = .hot
        default:
            self = .critical
        }
    }

    var label: String {
        switch self {
        case .idle:     return "Idle"
        case .normal:   return "Normal"
        case .busy:     return "Busy"
        case .hot:      return "Hot"
        case .critical: return "Critical"
        }
    }

    var color: Color {
        switch self {
        case .idle:     return .gray
        case .normal:   return .green
        case .busy:     return .blue
        case .hot:      return .orange
        case .critical: return .red
        }
    }
}