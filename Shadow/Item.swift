//
//  Item.swift
//  Shadow
//
//  Created by root_shine on 12/1/25.
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
