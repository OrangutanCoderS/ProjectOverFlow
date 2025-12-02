//
//  NetworkStatsStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/NetworkStatsStateManager.swift

import Foundation
import Combine

/// Stores high-level network stats for graphs and connection tables.
@MainActor
final class NetworkStatsStateManager: ObservableObject {

    static let shared = NetworkStatsStateManager()

    // Later: @Published var interfaceStats: [...]
    // Later: @Published var connectionList: [...]

    private init() {}

    func update(_ event: NetworkStatsEvent) {
        // TODO: map event into per-interface bandwidth series,
        // and maybe connection metadata for NetworkConnectionsView.
    }
}
