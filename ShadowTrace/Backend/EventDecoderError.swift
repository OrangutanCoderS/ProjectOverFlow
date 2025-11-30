//
//  EventDecoderError.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

/// Errors that can occur while decoding backend events.
enum EventDecoderError: Error {
    case invalidJSON(underlying: Error)
    case unknownType(String)
    case payloadDecodeFailed(type: String, underlying: Error)
}
