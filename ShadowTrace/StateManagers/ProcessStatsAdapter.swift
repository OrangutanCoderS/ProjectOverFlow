//
//  ProcessStatsAdapter.swift
//  ShadowTrace
//

import Foundation
import Combine

// MARK: - Clean UI Model For Processes

/// UI-facing model used by *all* process views.
struct UIProcessModel: Identifiable {
    let id = UUID()

    let pid: Int
    let name: String
    let user: String

    var cpu: Double          // smoothed %
    var memMb: Double        // in MB
    var isAlive: Bool

    var parentPid: Int?
    var children: [Int]

    var lastUpdated: TimeInterval = Date().timeIntervalSince1970
}

// MARK: - ProcessStatsAdapter

@MainActor
enum ProcessStatsAdapter {

    /// Global rolling PID map.
    private static var processMap: [Int: UIProcessModel] = [:]

    /// Smoothing factors.
    private static let cpuSmooth: Double = 0.6

    /// Anomaly detection thresholds.
    private static let highCpuThreshold = 80.0      // %
    private static let highMemThreshold = 1024.0    // MB
    private static let forkStormThreshold = 20      // children

    /// Main entry point from BackendStateManager.
    static func handle(_ event: ProcessSnapshotEvent, timestamp: TimeInterval) {

        var newMap: [Int: UIProcessModel] = [:]

        // Build children mapping for fork detection.
        var parentToChildren: [Int: [Int]] = [:]

        for entry in event.processes {
            if let parent = extractParentPid(from: entry) {
                parentToChildren[parent, default: []].append(entry.pid)
            }
        }

        // Build stable UI models
        for entry in event.processes {
            let pid = entry.pid
            let prev = processMap[pid]

            let newCpu = clamp(entry.cpu, between: 0, and: 100)
            let cpu = prev == nil
                ? newCpu
                : prev!.cpu * cpuSmooth + newCpu * (1 - cpuSmooth)

            let mem = entry.memMb

            let model = UIProcessModel(
                pid: pid,
                name: entry.name,
                user: entry.user,
                cpu: cpu,
                memMb: mem,
                isAlive: true,
                parentPid: extractParentPid(from: entry),
                children: parentToChildren[pid] ?? [],
                lastUpdated: timestamp
            )

            newMap[pid] = model

            // Anomaly detection
            detectHighCpu(model)
            detectHighMemory(model)
            detectZombie(model)
        }

        // Mark dead processes (fade-out)
        for (pid, oldProc) in processMap {
            if newMap[pid] == nil {
                let deadModel = UIProcessModel(
                    pid: oldProc.pid,
                    name: oldProc.name,
                    user: oldProc.user,
                    cpu: oldProc.cpu * 0.3,
                    memMb: oldProc.memMb,
                    isAlive: false,
                    parentPid: oldProc.parentPid,
                    children: oldProc.children,
                    lastUpdated: timestamp
                )
                newMap[pid] = deadModel
            }
        }

        // Update global map
        processMap = newMap

        // Fork bomb detection
        detectForkStorm(parentToChildren)

        // Export sorted list to ProcessStateManager
        let sortedList = newMap.values
            .sorted { a, b in
                if a.isAlive != b.isAlive { return a.isAlive && !b.isAlive }
                if a.cpu != b.cpu { return a.cpu > b.cpu }
                return a.name < b.name
            }

        ProcessStateManager.shared.updateProcesses(sortedList)
    }

    // MARK: - Parent PID extractor (stub)

    private static func extractParentPid(from entry: ProcessInfoEventEntry) -> Int? {
        // Future: backend will provide ppid; for now, we don't have it.
        return nil
    }

    // MARK: - Anomaly Detection

    private static func detectHighCpu(_ p: UIProcessModel) {
        if p.cpu > highCpuThreshold && p.isAlive {
            raiseAlert(
                title: "High CPU Usage",
                message: "\(p.name) (PID \(p.pid)) is at \(Int(p.cpu))% CPU."
            )
        }
    }

    private static func detectHighMemory(_ p: UIProcessModel) {
        if p.memMb > highMemThreshold && p.isAlive {
            raiseAlert(
                title: "High Memory Usage",
                message: "\(p.name) (PID \(p.pid)) is using \(Int(p.memMb)) MB."
            )
        }
    }

    private static func detectZombie(_ p: UIProcessModel) {
        // Naive zombie: alive + ~0 CPU + tiny memory.
        if p.isAlive && p.cpu < 0.1 && p.memMb < 10 {
            raiseAlert(
                title: "Zombie Process Suspected",
                message: "\(p.name) (PID \(p.pid)) appears defunct (0% CPU, <10MB)."
            )
        }
    }

    private static func detectForkStorm(_ tree: [Int: [Int]]) {
        for (parent, children) in tree {
            if children.count > forkStormThreshold {
                raiseAlert(
                    title: "Fork Storm Detected",
                    message: "PID \(parent) spawned \(children.count) children."
                )
            }
        }
    }

    // MARK: - Alert bridge → BackendStateManager

    private static func raiseAlert(
        title: String,
        message: String,
        level: AlertItem.Level = .warning
    ) {
        let alert = AlertItem(
            title: title,
            message: message,
            level: level
        )
        BackendStateManager.shared.pushGlobalAlert(alert)
    }

    // MARK: - Utilities

    private static func clamp(_ value: Double, between low: Double, and high: Double) -> Double {
        return min(max(value, low), high)
    }
}
