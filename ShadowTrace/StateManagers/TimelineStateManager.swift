//
//  TimelineStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// StateManagers/TimelineStateManager.swift

import Foundation
import Combine

/// Owns the trace timeline and replay coordination.
@MainActor
final class TimelineStateManager: ObservableObject {

    static let shared = TimelineStateManager()

    // Later: @Published var windows: [ /* TimelineWindowModel */ ] = []
    // Later: @Published var activeWindowId: UUID?

    private init() {}

    func prepareForReplay() {
        // TODO:
        // - Clear transient realtime buffers if needed
        // - Prepare data structures for incoming timeline chunks
    }

    func exitReplay() {
        // TODO:
        // - Return to live mode, maybe discard replay-only buffers
    }

    func appendChunk(_ chunk: TimelineChunkEvent) {
        // TODO:
        // - Add chunk to the appropriate window
        // - Rebuild derived indices or summaries
    }

    func updateReplayProgress(_ event: ReplayProgressEvent) {
        // TODO:
        // - When ReplayProgressEvent gets fields, update progress indicators.
    }

    func reset() {
        // TODO:
        // - Full wipe of timeline state (e.g. when switching sessions)
    }
}
