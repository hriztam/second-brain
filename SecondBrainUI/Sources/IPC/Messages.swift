//
//  Messages.swift
//  SecondBrain
//
//  IPC message types matching the Rust daemon protocol
//

import Foundation

// MARK: - Daemon Mode

enum DaemonMode: String, Codable {
    case idle
    case dictation
    case intelligent
    case agent
    
    var displayName: String {
        switch self {
        case .idle: return "Idle"
        case .dictation: return "Dictation"
        case .intelligent: return "Intelligent"
        case .agent: return "Agent"
        }
    }
}

// MARK: - Requests (UI → Daemon)

enum Request: Encodable {
    case getStatus
    case setMode(mode: DaemonMode)
    case ping
    
    private enum CodingKeys: String, CodingKey {
        case type, mode
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .getStatus:
            try container.encode("get_status", forKey: .type)
        case .setMode(let mode):
            try container.encode("set_mode", forKey: .type)
            try container.encode(mode, forKey: .mode)
        case .ping:
            try container.encode("ping", forKey: .type)
        }
    }
}

// MARK: - Responses (Daemon → UI)

enum Response: Decodable {
    case status(DaemonStatus)
    case modeChange(mode: DaemonMode, active: Bool)
    case pong
    case error(code: String, message: String)
    
    private enum CodingKeys: String, CodingKey {
        case type, mode, active, code, message
        case version, hotkeyRegistered = "hotkey_registered", uptimeSecs = "uptime_secs"
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        
        switch type {
        case "status":
            let status = try DaemonStatus(from: decoder)
            self = .status(status)
        case "mode_change":
            let mode = try container.decode(DaemonMode.self, forKey: .mode)
            let active = try container.decode(Bool.self, forKey: .active)
            self = .modeChange(mode: mode, active: active)
        case "pong":
            self = .pong
        case "error":
            let code = try container.decode(String.self, forKey: .code)
            let message = try container.decode(String.self, forKey: .message)
            self = .error(code: code, message: message)
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown response type: \(type)"
            )
        }
    }
}

// MARK: - Daemon Status

struct DaemonStatus: Decodable {
    let version: String
    let mode: DaemonMode
    let hotkeyRegistered: Bool
    let uptimeSecs: UInt64
    
    private enum CodingKeys: String, CodingKey {
        case type, version, mode
        case hotkeyRegistered = "hotkey_registered"
        case uptimeSecs = "uptime_secs"
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        version = try container.decode(String.self, forKey: .version)
        mode = try container.decode(DaemonMode.self, forKey: .mode)
        hotkeyRegistered = try container.decode(Bool.self, forKey: .hotkeyRegistered)
        uptimeSecs = try container.decode(UInt64.self, forKey: .uptimeSecs)
    }
}
