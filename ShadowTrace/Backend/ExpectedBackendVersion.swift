//
//  ExpectedBackendVersion.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// Backend/ExpectedBackendVersion.swift

import Foundation

/// Expected Rust OverFlow backend semantic version for this ShadowTrace build.
/// Wire this up to your Cargo.toml / build metadata later.
enum ExpectedBackendVersion {
    static let string: String = "0.1.0-dev"
}
