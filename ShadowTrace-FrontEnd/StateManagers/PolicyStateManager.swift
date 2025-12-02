//
//  PolicyStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/PolicyStateManager.swift

import Foundation
import Combine

/// Holds current policy tree / rule summaries for UI inspection.
@MainActor
final class PolicyStateManager: ObservableObject {

    static let shared = PolicyStateManager()

    // Later: @Published var policies: [ /* PolicyModel */ ] = []

    private init() {}

    func update(_ event: PolicyUpdateEvent) {
        // TODO:
        // - Apply incoming diffs to the policy list
        // - Keep track of active / disabled policies
    }

    func reset() {
        // TODO: clear cached policies when backend resets or config reloads.
    }
}
