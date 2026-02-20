import Foundation

enum ChatRole: String {
    case user
    case assistant
    case system
    case debug
}

struct ChatMessage: Identifiable, Equatable {
    let id: UUID
    let role: ChatRole
    let text: String
    let timestamp: Date

    init(id: UUID = UUID(), role: ChatRole, text: String, timestamp: Date = Date()) {
        self.id = id
        self.role = role
        self.text = text
        self.timestamp = timestamp
    }
}
