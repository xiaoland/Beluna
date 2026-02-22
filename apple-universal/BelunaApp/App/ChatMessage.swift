import Foundation

enum ChatRole: String {
    case user
    case assistant
    case system
    case debug
    case tool
}

struct ToolCallMessagePayload: Equatable {
    let cycleID: UInt64
    let stage: String
    let inputPayload: String
    let outputPayload: String
}

enum ChatMessageBody: Equatable {
    case text(String)
    case toolCall(ToolCallMessagePayload)
}

struct ChatMessage: Identifiable, Equatable {
    let id: UUID
    let role: ChatRole
    let body: ChatMessageBody
    let timestamp: Date

    init(id: UUID = UUID(), role: ChatRole, text: String, timestamp: Date = Date()) {
        self.id = id
        self.role = role
        self.body = .text(text)
        self.timestamp = timestamp
    }

    init(id: UUID = UUID(), toolCall: ToolCallMessagePayload, timestamp: Date = Date()) {
        self.id = id
        self.role = .tool
        self.body = .toolCall(toolCall)
        self.timestamp = timestamp
    }

    var text: String {
        switch body {
        case let .text(text):
            return text
        case let .toolCall(payload):
            return "[tool] cycle \(payload.cycleID) \(payload.stage)"
        }
    }
}
