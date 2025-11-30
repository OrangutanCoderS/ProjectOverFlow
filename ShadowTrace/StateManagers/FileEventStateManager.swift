//
//  FileEventStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import Combine

/// Central store for file activity events powering FileActivityView
/// and DashboardView summaries.
@MainActor
final class FileEventStateManager: ObservableObject {

    static let shared = FileEventStateManager()

    /// Rolling window of file events (prevents unbounded memory growth)
    @Published private(set) var events: [FileEvent] = []

    /// Maximum number of events to keep in memory
    private let maxInMemory = 5_000

    private init() {}

    /// Insert one new event coming from backend
    func insert(_ event: FileEvent) {
        events.append(event)
        if events.count > maxInMemory {
            let dropCount = events.count - maxInMemory
            events.removeFirst(dropCount)
        }
    }

    /// Reset window when replay mode starts or a new session begins
    func reset() {
        events.removeAll(keepingCapacity: true)
    }
}
