//
//  ProcessStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//


//
//  ProcessStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import Combine

/// UI-facing store for the current process list.
/// Filled by `ProcessStatsAdapter` when a `ProcessSnapshotEvent` arrives.
@MainActor
final class ProcessStateManager: ObservableObject {

    static let shared = ProcessStateManager()

    /// Current set of processes, sorted by whatever policy the adapter chose.
    @Published private(set) var processes: [UIProcessModel] = []

    private init() {}

    func updateProcesses(_ newProcesses: [UIProcessModel]) {
        processes = newProcesses
    }

    /// Number of tracked processes, used in the CPU dashboard card.
    var processCount: Int {
        processes.count
    }
}
