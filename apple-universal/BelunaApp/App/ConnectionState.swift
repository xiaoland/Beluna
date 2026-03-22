import SwiftUI

enum ConnectionState: String {
    case disconnected = "Disconnected"
    case connecting = "Connecting"
    case connected = "Connected"

    var tint: Color {
        switch self {
        case .connected:
            return .green
        case .connecting:
            return .orange
        case .disconnected:
            return .red
        }
    }
}

enum ReconnectStatus: Equatable {
    case idle
    case scheduled(attempt: Int, maxAttempts: Int, delaySeconds: Double)
    case exhausted(maxAttempts: Int)
}
