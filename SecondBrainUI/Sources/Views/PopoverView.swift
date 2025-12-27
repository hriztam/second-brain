//
//  PopoverView.swift
//  SecondBrain
//
//  Main popover content showing daemon status and mode controls
//

import SwiftUI

struct PopoverView: View {
    let isConnected: Bool
    let currentMode: DaemonMode
    let onModeChange: (DaemonMode) -> Void
    
    var body: some View {
        VStack(spacing: 16) {
            // Header with connection status
            HStack {
                Text("Second Brain")
                    .font(.headline)
                
                Spacer()
                
                ConnectionIndicator(isConnected: isConnected)
            }
            
            Divider()
            
            if isConnected {
                // Mode selector
                VStack(alignment: .leading, spacing: 12) {
                    Text("Mode")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                    
                    ModeButton(
                        title: "Dictation",
                        subtitle: "Low-latency transcription",
                        icon: "mic.fill",
                        isSelected: currentMode == .dictation,
                        action: { onModeChange(.dictation) }
                    )
                    
                    ModeButton(
                        title: "Intelligent",
                        subtitle: "LLM responses",
                        icon: "sparkles",
                        isSelected: currentMode == .intelligent,
                        action: { onModeChange(.intelligent) }
                    )
                    
                    ModeButton(
                        title: "Agent",
                        subtitle: "Multi-step tasks",
                        icon: "figure.walk",
                        isSelected: currentMode == .agent,
                        action: { onModeChange(.agent) }
                    )
                }
                
                Spacer()
                
                // Status footer
                HStack {
                    Text(currentMode == .idle ? "Ready" : "Mode: \(currentMode.displayName)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Spacer()
                    
                    Button("Quit") {
                        NSApplication.shared.terminate(nil)
                    }
                    .buttonStyle(.plain)
                    .foregroundColor(.secondary)
                }
            } else {
                // Disconnected state
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundColor(.orange)
                    
                    Text("Daemon not connected")
                        .font(.headline)
                    
                    Text("Attempting to reconnect...")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    ProgressView()
                        .progressViewStyle(.circular)
                        .scaleEffect(0.8)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .padding()
        .frame(width: 300, height: 380)
    }
}

// MARK: - Connection Indicator

struct ConnectionIndicator: View {
    let isConnected: Bool
    
    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(isConnected ? Color.green : Color.red)
                .frame(width: 8, height: 8)
            
            Text(isConnected ? "Connected" : "Disconnected")
                .font(.caption)
                .foregroundColor(.secondary)
        }
    }
}

// MARK: - Mode Button

struct ModeButton: View {
    let title: String
    let subtitle: String
    let icon: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.title2)
                    .frame(width: 32)
                    .foregroundColor(isSelected ? .white : .accentColor)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .fontWeight(.medium)
                        .foregroundColor(isSelected ? .white : .primary)
                    
                    Text(subtitle)
                        .font(.caption)
                        .foregroundColor(isSelected ? .white.opacity(0.8) : .secondary)
                }
                
                Spacer()
                
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.white)
                }
            }
            .padding(12)
            .background(isSelected ? Color.accentColor : Color.gray.opacity(0.1))
            .cornerRadius(8)
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Preview

#Preview {
    PopoverView(
        isConnected: true,
        currentMode: .dictation,
        onModeChange: { _ in }
    )
}
