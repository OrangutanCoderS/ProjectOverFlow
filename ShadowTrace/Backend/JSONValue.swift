//
//  JSONValue.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import Foundation

/// Generic JSON tree used to store the backend payload
/// before re-decoding into strongly-typed Swift structs.
///
/// Compatible with BOTH:
/// - ShadowTrace native events
/// - Rust AnyEvent JSON output
enum JSONValue: Codable, Equatable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        // null
        if container.decodeNil() {
            self = .null
            return
        }

        // bool
        if let b = try? container.decode(Bool.self) {
            self = .bool(b)
            return
        }

        // Rust sometimes uses Int or UInt — decode safely
        if let i = try? container.decode(Int.self) {
            self = .number(Double(i))
            return
        }

        if let u = try? container.decode(UInt.self) {
            self = .number(Double(u))
            return
        }

        // number (Double)
        if let n = try? container.decode(Double.self) {
            self = .number(n)
            return
        }

        // string
        if let s = try? container.decode(String.self) {
            self = .string(s)
            return
        }

        // object
        if let obj = try? container.decode([String: JSONValue].self) {
            self = .object(obj)
            return
        }

        // array
        if let arr = try? container.decode([JSONValue].self) {
            self = .array(arr)
            return
        }

        // fallback
        self = .null
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .string(let s):
            try container.encode(s)

        case .number(let n):
            try container.encode(n)

        case .bool(let b):
            try container.encode(b)

        case .object(let obj):
            try container.encode(obj)

        case .array(let arr):
            try container.encode(arr)

        case .null:
            try container.encodeNil()
        }
    }
}
