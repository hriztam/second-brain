//
//  DaemonClient.swift
//  SecondBrain
//
//  Unix domain socket client for daemon communication
//

import Foundation
import Network

/// Client for communicating with the second-brain daemon over Unix domain socket
class DaemonClient {
    private var connection: NWConnection?
    private let socketPath: String
    private let queue = DispatchQueue(label: "com.secondbrain.daemon-client")
    
    init() {
        // Socket path matches daemon config
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        self.socketPath = "\(home)/.local/share/second-brain/daemon.sock"
    }
    
    // MARK: - Connection Management
    
    func connect() async throws {
        return try await withCheckedThrowingContinuation { continuation in
            let endpoint = NWEndpoint.unix(path: socketPath)
            
            // Use TCP-like parameters for stream-based Unix socket
            let parameters = NWParameters.tcp
            parameters.allowLocalEndpointReuse = true
            
            // Disable TLS for local socket
            parameters.defaultProtocolStack.applicationProtocols.removeAll()
            
            connection = NWConnection(to: endpoint, using: parameters)
            
            var didResume = false
            connection?.stateUpdateHandler = { [weak self] state in
                guard !didResume else { return }
                switch state {
                case .ready:
                    didResume = true
                    continuation.resume()
                case .failed(let error):
                    didResume = true
                    continuation.resume(throwing: error)
                case .cancelled:
                    didResume = true
                    continuation.resume(throwing: DaemonClientError.connectionCancelled)
                case .waiting(let error):
                    // Log waiting state but don't fail yet
                    print("Connection waiting: \(error)")
                default:
                    break
                }
            }
            
            connection?.start(queue: queue)
        }
    }
    
    func disconnect() {
        connection?.cancel()
        connection = nil
    }
    
    // MARK: - Request/Response
    
    private func send<T: Encodable>(_ request: T) async throws {
        guard let connection = connection else {
            throw DaemonClientError.notConnected
        }
        
        let encoder = JSONEncoder()
        let data = try encoder.encode(request)
        
        // Length-prefixed: 4-byte little-endian length + JSON
        var length = UInt32(data.count).littleEndian
        var frame = Data(bytes: &length, count: 4)
        frame.append(data)
        
        return try await withCheckedThrowingContinuation { continuation in
            connection.send(content: frame, completion: .contentProcessed { error in
                if let error = error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume()
                }
            })
        }
    }
    
    private func receive() async throws -> Response {
        guard let connection = connection else {
            throw DaemonClientError.notConnected
        }
        
        // Read 4-byte length prefix
        let lengthData: Data = try await withCheckedThrowingContinuation { continuation in
            connection.receive(minimumIncompleteLength: 4, maximumLength: 4) { data, _, _, error in
                if let error = error {
                    continuation.resume(throwing: error)
                } else if let data = data {
                    continuation.resume(returning: data)
                } else {
                    continuation.resume(throwing: DaemonClientError.noData)
                }
            }
        }
        
        let length = lengthData.withUnsafeBytes { $0.load(as: UInt32.self).littleEndian }
        
        // Read message body
        let bodyData: Data = try await withCheckedThrowingContinuation { continuation in
            connection.receive(minimumIncompleteLength: Int(length), maximumLength: Int(length)) { data, _, _, error in
                if let error = error {
                    continuation.resume(throwing: error)
                } else if let data = data {
                    continuation.resume(returning: data)
                } else {
                    continuation.resume(throwing: DaemonClientError.noData)
                }
            }
        }
        
        let decoder = JSONDecoder()
        return try decoder.decode(Response.self, from: bodyData)
    }
    
    // MARK: - High-Level API
    
    func getStatus() async throws -> DaemonStatus {
        try await send(Request.getStatus)
        let response = try await receive()
        
        switch response {
        case .status(let status):
            return status
        case .error(let code, let message):
            throw DaemonClientError.daemonError(code: code, message: message)
        default:
            throw DaemonClientError.unexpectedResponse
        }
    }
    
    func setMode(_ mode: DaemonMode) async throws {
        try await send(Request.setMode(mode: mode))
        let response = try await receive()
        
        switch response {
        case .modeChange:
            return
        case .error(let code, let message):
            throw DaemonClientError.daemonError(code: code, message: message)
        default:
            throw DaemonClientError.unexpectedResponse
        }
    }
    
    func ping() async throws {
        try await send(Request.ping)
        let response = try await receive()
        
        switch response {
        case .pong:
            return
        case .error(let code, let message):
            throw DaemonClientError.daemonError(code: code, message: message)
        default:
            throw DaemonClientError.unexpectedResponse
        }
    }
}

// MARK: - Errors

enum DaemonClientError: LocalizedError {
    case notConnected
    case connectionCancelled
    case noData
    case unexpectedResponse
    case daemonError(code: String, message: String)
    
    var errorDescription: String? {
        switch self {
        case .notConnected:
            return "Not connected to daemon"
        case .connectionCancelled:
            return "Connection was cancelled"
        case .noData:
            return "No data received"
        case .unexpectedResponse:
            return "Unexpected response from daemon"
        case .daemonError(let code, let message):
            return "Daemon error [\(code)]: \(message)"
        }
    }
}
