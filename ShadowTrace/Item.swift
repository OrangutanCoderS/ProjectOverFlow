//
//  Item.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation
import SwiftData

@Model
final class Item {
    var timestamp: Date
    
    init(timestamp: Date) {
        self.timestamp = timestamp
    }
}
