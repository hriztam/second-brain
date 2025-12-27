//
//  SecondBrainApp.swift
//  SecondBrain
//
//  Background daemon communication + menu bar integration
//

import SwiftUI

@main
struct SecondBrainApp: App {
    // AppDelegate handles NSStatusItem and daemon connection
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        // No window - menu bar only app
        Settings {
            EmptyView()
        }
    }
}
