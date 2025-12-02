//
//  ProcessTrackerAdapter.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/ProcessTrackerAdapter.swift

import Foundation
import Combine

/// Keeps a live, deduplicated view of all processes for the Process tab.
@MainActor
final class ProcessTrackerAdapter: ObservableObject {

    static let shared = ProcessTrackerAdapter()

    // Later: @Published var processes: [ProcessInfoModel] = []

    private init() {}

    func updateProcesses(_ snapshot: ProcessSnapshotEvent) {
        // TODO:
        // - Convert snapshot into [ProcessInfoModel]
        // - Diff & update published list
        // - Maintain CPU / memory trend buffers for each PID if needed
    }
}
