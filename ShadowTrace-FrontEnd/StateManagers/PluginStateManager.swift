//
//  PluginStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/PluginStateManager.swift

import Foundation
import Combine

/// Keeps track of plugin metadata and recent triggers for the Plugin views.
@MainActor
final class PluginStateManager: ObservableObject {

    static let shared = PluginStateManager()

    // Later: @Published var plugins: [PluginModel] = []
    // Later: @Published var recentTriggers: [PluginTriggerEvent] = []

    private init() {}

    func registerTrigger(_ event: PluginTriggerEvent) {
        // TODO:
        // - Append to recentTriggers with size bound
        // - Update plugin health / status indicators if needed
    }

    func reset() {
        // TODO: clear transient plugin trigger state on replay / session reset
    }
}
