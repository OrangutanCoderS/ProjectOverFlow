//
//  PolicyAuditManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/PolicyAuditManager.swift

import Foundation
import Combine

/// Manages the audit log stream for policy decisions & actions.
@MainActor
final class PolicyAuditManager: ObservableObject {

    static let shared = PolicyAuditManager()

    // Later: @Published var entries: [AuditLogEntry] = []

    private init() {}

    func insert(_ entry: AuditLogEntry) {
        // TODO:
        // - Append to entries with retention policy
        // - Group by policy / plugin / category if needed
    }

    func reset() {
        // TODO: clear audit log in replay or when user manually clears.
    }
}
