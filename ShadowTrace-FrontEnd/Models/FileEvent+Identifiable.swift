//
//  FileEvent+Identifiable.swift
//  ShadowTrace
//
//  Created by root_shine on 11/29/25.
//

import Foundation

/// Make backend `FileEvent` usable in SwiftUI `ForEach` without
/// polluting the core model with UI-specific concerns.
extension FileEvent: Identifiable {
    var id: String {
        // Stable-ish composite key: good enough for UI diffing.
        "\(pid)|\(processName)|\(path)|\(op)"
    }
}
