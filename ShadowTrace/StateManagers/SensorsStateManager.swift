//
//  SensorsStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import Combine

/// Tracks thermal, battery, and other hardware sensor data.
@MainActor
final class SensorsStateManager: ObservableObject {

    static let shared = SensorsStateManager()

    /// Flat list of all sensor readings coming from the backend.
    @Published var sensors: [SensorReading] = []

    private init() {}

    /// Upsert a sensor reading from a backend event.
    func update(_ event: SensorUpdateEvent) {
        let now = Date()

        let reading = SensorReading(
            key: event.key,
            label: event.label,
            value: event.value,
            unit: event.unit ?? "",
            category: event.category,
            sortOrder: event.sortOrder ?? event.category.sortOrder,
            lastUpdated: now
        )

        if let idx = sensors.firstIndex(where: { $0.key == reading.key }) {
            // Replace existing reading
            sensors[idx] = reading
        } else {
            sensors.append(reading)
        }
    }

    /// Convenience for batch updates (e.g. snapshot from backend).
    func replaceAll(with events: [SensorUpdateEvent]) {
        let now = Date()
        self.sensors = events.map { e in
            SensorReading(
                key: e.key,
                label: e.label,
                value: e.value,
                unit: e.unit ?? "",
                category: e.category,
                sortOrder: e.sortOrder ?? e.category.sortOrder,
                lastUpdated: now
            )
        }
    }
}
