//
//  ContentView.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

import SwiftUI

struct ContentView: View {
    var body: some View {
        NavigationSplitView {
            // Sidebar — simple for now
            List {
                NavigationLink("Dashboard") {
                    DashboardView()
                }
                .tag("dashboard")
            }
            .navigationSplitViewColumnWidth(min: 180, ideal: 220)

        } detail: {
            DashboardView()
                .frame(minWidth: 1100, minHeight: 700)
                .navigationTitle("")       // macOS-friendly "hide"
                .toolbar {                 // ensures no unwanted toolbar text
                    ToolbarItem(placement: .principal) {
                        EmptyView()
                    }
                }
        }
    }
}

#Preview {
    ContentView()
}
