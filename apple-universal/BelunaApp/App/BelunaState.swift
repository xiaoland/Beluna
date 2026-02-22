import SwiftUI

enum BelunaState: String {
    case unknown = "Unknown"
    case hibernate = "Hibernate"
    case awake = "Awake"

    var tint: Color {
        switch self {
        case .unknown:
            return .gray
        case .hibernate:
            return .orange
        case .awake:
            return .green
        }
    }
}
