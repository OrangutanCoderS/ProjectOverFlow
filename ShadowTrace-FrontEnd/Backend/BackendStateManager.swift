//
//  BackendStateManager.swift
//  ShadowTrace
//
//  Created by root_shine on 11/28/25.
//

// Backend/BackendStateManager.swift

import Foundation
import Combine

/// Global coordinator for everything coming from the OverFlow backend.
/// - Receives high-level `BackendEvent` from BackendClient/EventDecoder
/// - Routes events to domain-specific state managers/adapters
/// - Tracks global connection / replay / alert state for the UI
@MainActor
final class BackendStateManager: ObservableObject {
    
    static let shared = BackendStateManager()
    
    // MARK: - Global UI State
    
    @Published private(set) var backendVersion: String = "unknown"
    @Published private(set) var connectionStatus: ConnectionStatus = .disconnected
    @Published private(set) var lastEventTimestamp: TimeInterval = 0
    
    @Published private(set) var inReplayMode: Bool = false
    @Published private(set) var replayProgress: Double = 0.0
    
    @Published private(set) var globalAlerts: [AlertItem] = []
    
    // MARK: - Init
    
    private init() {}
    
    // MARK: - External control hooks
    
    func updateConnectionStatus(_ status: ConnectionStatus) {
        connectionStatus = status
    }
    
    func setBackendVersion(_ version: String) {
        backendVersion = version
        
        // Basic version check hook (can be expanded later)
        if version != ExpectedBackendVersion.string {
            let alert = AlertItem(
                title: "Backend Version Mismatch",
                message: "UI expects \(ExpectedBackendVersion.string), but backend reports \(version). Some features may not work correctly.",
                level: .warning
            )
            pushGlobalAlert(alert)
        }
    }
    
    func enterReplayMode() {
        guard !inReplayMode else { return }
        inReplayMode = true
        replayProgress = 0.0
        TimelineStateManager.shared.prepareForReplay()
    }
    
    func exitReplayMode() {
        guard inReplayMode else { return }
        inReplayMode = false
        replayProgress = 0.0
        TimelineStateManager.shared.exitReplay()
    }
    
    // MARK: - Alert Management
    
    func pushGlobalAlert(_ alert: AlertItem) {
        globalAlerts.append(alert)
    }
    
    func clearAlert(_ id: UUID) {
        globalAlerts.removeAll { $0.id == id }
    }
    
    func clearAllAlerts() {
        globalAlerts.removeAll()
    }
    
    // MARK: - Core Dispatcher
    
    /// Central entry point from BackendClient.
    /// All decoded backend events pass through here.
    // MARK: - Core Dispatcher
    
    func handle(event: BackendEvent) {
        lastEventTimestamp = Date().timeIntervalSince1970
        
        switch event {
            
            // --- CPU ---
        case .cpuStats(let payload, _):
            SystemStatsAdapter.shared.updateCpu(payload)
            
            // --- Memory / System Stats ---
        case .memoryStats(let payload, _):
            SystemStatsAdapter.shared.updateMemory(payload)
            
            // --- GPU ---
        case .gpuStats(let payload, _):
            GpuStatsStateManager.shared.update(payload)
            
            // --- Battery ---
        case .batteryStats(let evt, _):
            BatteryStatsAdapter.shared.update(
                levelPercent: evt.percentage,
                isCharging: evt.charging,
                onACPower: false,
                cycleCount: evt.cycleCount,
                temperatureC: evt.temperatureC,
                powerWatts: nil
            )
            
            // --- Thermal ---
        case .thermalStats(let payload, _):
            ThermalStateManager.shared.update(payload)
            
            // --- Processes ---
        case .processSnapshot(let payload, _):
            ProcessTrackerAdapter.shared.updateProcesses(payload)
            
            // --- File events ---
        case .fileEvent(let payload, _):
            FileEventStateManager.shared.insert(payload)
            
            // --- Sensors (deprecated, but supported) ---
        case .sensorUpdate(let payload, _):
            SensorsStateManager.shared.update(payload)
            
            // --- Network ---
        case .networkStats(let payload, _):
            NetworkStatsStateManager.shared.update(payload)
            
            // --- Plugins ---
        case .pluginTrigger(let payload, _):
            PluginStateManager.shared.registerTrigger(payload)
            
            // --- Policies ---
        case .policyUpdate(let payload, _):
            PolicyStateManager.shared.update(payload)
            
            // --- Audit Log ---
        case .auditLogEntry(let entry, _):
            PolicyAuditManager.shared.insert(entry)
            
            // --- Timeline Replay ---
        case .timelineChunk(let chunk, _):
            if !inReplayMode {
                inReplayMode = true
                TimelineStateManager.shared.prepareForReplay()
            }
            TimelineStateManager.shared.appendChunk(chunk)
            
        case .replayProgress(let progress, _):
            inReplayMode = true
            TimelineStateManager.shared.updateReplayProgress(progress)
            
            // --- Version ---
        case .versionInfo(let versionEvent, _):
            setBackendVersion(versionEvent.version)
            
            // --- Errors ---
        case .errorEvent(let error, _):
            let alert = AlertItem(
                title: "Backend Error",
                message: error.message,
                level: .error
            )
            pushGlobalAlert(alert)
            
            // --- Unknown ---
        case .unknown(let raw, _):
            print("Unknown backend event type: \(raw.rawType)")
        }
    }
}
