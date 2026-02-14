import SwiftUI

enum BelunaState: String {
    case unknown = "Unknown"
    case sleeping = "Sleeping"
    case awake = "Awake"

    var tint: Color {
        switch self {
        case .unknown:
            return .gray
        case .sleeping:
            return .orange
        case .awake:
            return .green
        }
    }
}
