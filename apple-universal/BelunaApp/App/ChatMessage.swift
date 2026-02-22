import Foundation

enum ChatRole: String {
    case user
    case assistant
    case system
    case debug
    case organActivity
}

struct OrganActivityMessagePayload: Identifiable, Equatable {
    let id: UUID
    let stage: String
    let inputPayload: String
    let outputPayload: String
    let timestamp: Date

    init(
        id: UUID = UUID(),
        stage: String,
        inputPayload: String,
        outputPayload: String,
        timestamp: Date
    ) {
        self.id = id
        self.stage = stage
        self.inputPayload = inputPayload
        self.outputPayload = outputPayload
        self.timestamp = timestamp
    }
}

struct CortexCycleMessagePayload: Equatable {
    let cycleID: UInt64
    var organActivityMessages: [OrganActivityMessagePayload]
}

enum ChatMessageBody: Equatable {
    case text(String)
    case cortexCycle(CortexCycleMessagePayload)
}

struct ChatMessage: Identifiable, Equatable {
    let id: UUID
    let role: ChatRole
    var body: ChatMessageBody
    var timestamp: Date

    init(id: UUID = UUID(), role: ChatRole, text: String, timestamp: Date = Date()) {
        self.id = id
        self.role = role
        self.body = .text(text)
        self.timestamp = timestamp
    }

    init(id: UUID = UUID(), cortexCycle: CortexCycleMessagePayload, timestamp: Date = Date()) {
        self.id = id
        self.role = .organActivity
        self.body = .cortexCycle(cortexCycle)
        self.timestamp = timestamp
    }

    var text: String {
        switch body {
        case let .text(text):
            return text
        case let .cortexCycle(payload):
            return "[cortex cycle] \(payload.cycleID), \(payload.organActivityMessages.count) organ activity message(s)"
        }
    }
}
