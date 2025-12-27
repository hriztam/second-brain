//
//  AppDelegate.swift
//  SecondBrain
//
//  Manages NSStatusItem (menu bar icon) and popover lifecycle
//

import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    // Menu bar status item
    private var statusItem: NSStatusItem!
    
    // Popover for main UI
    private var popover: NSPopover!
    
    // Daemon connection client
    private var daemonClient: DaemonClient!
    
    // Connection state for UI updates
    @Published var isConnected: Bool = false
    @Published var currentMode: DaemonMode = .idle
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Create daemon client
        daemonClient = DaemonClient()
        
        // Setup menu bar icon
        setupStatusItem()
        
        // Setup popover
        setupPopover()
        
        // Connect to daemon
        Task {
            await connectToDaemon()
        }
        
        // Hide dock icon - menu bar only
        NSApp.setActivationPolicy(.accessory)
    }
    
    // MARK: - Status Item Setup
    
    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        if let button = statusItem.button {
            // Use SF Symbol for menu bar icon
            button.image = NSImage(systemSymbolName: "brain.head.profile", accessibilityDescription: "Second Brain")
            button.action = #selector(togglePopover)
            button.target = self
        }
    }
    
    // MARK: - Popover Setup
    
    private func setupPopover() {
        popover = NSPopover()
        popover.contentSize = NSSize(width: 320, height: 400)
        popover.behavior = .transient
        popover.animates = true
        popover.contentViewController = NSHostingController(
            rootView: PopoverView(
                isConnected: isConnected,
                currentMode: currentMode,
                onModeChange: { [weak self] mode in
                    Task {
                        await self?.setMode(mode)
                    }
                }
            )
        )
    }
    
    // MARK: - Popover Actions
    
    @objc private func togglePopover() {
        if let button = statusItem.button {
            if popover.isShown {
                popover.performClose(nil)
            } else {
                popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
                
                // Bring app to front
                NSApp.activate(ignoringOtherApps: true)
            }
        }
    }
    
    func showPopover() {
        if let button = statusItem.button {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            NSApp.activate(ignoringOtherApps: true)
        }
    }
    
    func hidePopover() {
        popover.performClose(nil)
    }
    
    // MARK: - Daemon Communication
    
    private func connectToDaemon() async {
        do {
            try await daemonClient.connect()
            await MainActor.run {
                isConnected = true
                updatePopoverState()
            }
            
            // Request initial status
            let status = try await daemonClient.getStatus()
            await MainActor.run {
                currentMode = status.mode
                updatePopoverState()
            }
        } catch {
            await MainActor.run {
                isConnected = false
                updatePopoverState()
            }
            print("Failed to connect to daemon: \(error)")
            
            // Retry connection after delay
            try? await Task.sleep(nanoseconds: 5_000_000_000) // 5 seconds
            await connectToDaemon()
        }
    }
    
    private func setMode(_ mode: DaemonMode) async {
        do {
            try await daemonClient.setMode(mode)
            await MainActor.run {
                currentMode = mode
                updatePopoverState()
            }
        } catch {
            print("Failed to set mode: \(error)")
        }
    }
    
    @MainActor
    private func updatePopoverState() {
        // Recreate popover content with updated state
        popover.contentViewController = NSHostingController(
            rootView: PopoverView(
                isConnected: isConnected,
                currentMode: currentMode,
                onModeChange: { [weak self] mode in
                    Task {
                        await self?.setMode(mode)
                    }
                }
            )
        )
        
        // Update status item appearance based on connection
        if let button = statusItem.button {
            let symbolName = isConnected ? "brain.head.profile" : "brain.head.profile.slash"
            button.image = NSImage(systemSymbolName: symbolName, accessibilityDescription: "Second Brain")
        }
    }
}
