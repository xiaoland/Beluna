import Foundation

enum ChatRole: String, Codable {
    case user
    case assistant
    case system
}

enum ChatMessageSignalOrigin: String, Codable {
    case local
    case sense
    case act
}

struct ChatMessage: Identifiable, Equatable, Codable {
    let id: UUID
    let role: ChatRole
    let signalOrigin: ChatMessageSignalOrigin
    let text: String
    let timestamp: Date

    init(
        id: UUID = UUID(),
        role: ChatRole,
        signalOrigin: ChatMessageSignalOrigin = .local,
        text: String,
        timestamp: Date = Date()
    ) {
        self.id = id
        self.role = role
        self.signalOrigin = signalOrigin
        self.text = text
        self.timestamp = timestamp
    }
}
