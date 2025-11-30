//
//  ProcessDetailTab.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessDetailTab.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// All tabs in the per-process inspector.
/// Lightweight, no backend dependency.
enum ProcessDetailTab: Hashable, CaseIterable {
    case cpu
    case memory
    case disk
    case network
    case energy
    case meta

    var title: String {
        switch self {
        case .cpu:     return "CPU"
        case .memory:  return "Memory"
        case .disk:    return "Disk"
        case .network: return "Network"
        case .energy:  return "Energy"
        case .meta:    return "Meta"
        }
    }

    var systemImage: String {
        switch self {
        case .cpu:     return "cpu"
        case .memory:  return "memorychip"
        case .disk:    return "externaldrive"
        case .network: return "network"
        case .energy:  return "bolt.fill"
        case .meta:    return "info.circle"
        }
    }
}