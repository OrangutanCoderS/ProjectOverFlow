//
//  AlertItem.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// Models/UI/AlertItem.swift

import Foundation

struct AlertItem: Identifiable, Equatable {
    enum Level {
        case info
        case warning
        case error
        case critical
    }

    let id: UUID
    let title: String
    let message: String
    let level: Level
    let timestamp: Date

    init(
        id: UUID = UUID(),
        title: String,
        message: String,
        level: Level,
        timestamp: Date = Date()
    ) {
        self.id = id
        self.title = title
        self.message = message
        self.level = level
        self.timestamp = timestamp
    }
}
