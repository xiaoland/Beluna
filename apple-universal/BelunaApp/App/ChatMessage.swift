import Foundation

enum ChatRole: String, Codable {
    case user
    case assistant
    case system
    case debug
    case organActivity
}

enum ChatMessageSignalOrigin: String, Codable {
    case local
    case sense
    case act
    case observability
}

struct OrganActivityMessagePayload: Identifiable, Equatable, Codable {
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

struct CortexCycleMessagePayload: Equatable, Codable {
    let cycleID: UInt64
    let awakeSequence: UInt64?
    var organActivityMessages: [OrganActivityMessagePayload]
}

enum ChatMessageBody: Equatable, Codable {
    case text(String)
    case cortexCycle(CortexCycleMessagePayload)

    private enum CodingKeys: String, CodingKey {
        case kind
        case text
        case cortexCycle
    }

    private enum Kind: String, Codable {
        case text
        case cortexCycle
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        switch try container.decode(Kind.self, forKey: .kind) {
        case .text:
            self = .text(try container.decode(String.self, forKey: .text))
        case .cortexCycle:
            self = .cortexCycle(try container.decode(CortexCycleMessagePayload.self, forKey: .cortexCycle))
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case let .text(text):
            try container.encode(Kind.text, forKey: .kind)
            try container.encode(text, forKey: .text)
        case let .cortexCycle(payload):
            try container.encode(Kind.cortexCycle, forKey: .kind)
            try container.encode(payload, forKey: .cortexCycle)
        }
    }
}

struct ChatMessage: Identifiable, Equatable, Codable {
    let id: UUID
    let role: ChatRole
    let signalOrigin: ChatMessageSignalOrigin
    var body: ChatMessageBody
    var timestamp: Date

    private enum CodingKeys: String, CodingKey {
        case id
        case role
        case signalOrigin
        case body
        case timestamp
    }

    init(id: UUID = UUID(), role: ChatRole, signalOrigin: ChatMessageSignalOrigin = .local, text: String, timestamp: Date = Date()) {
        self.id = id
        self.role = role
        self.signalOrigin = signalOrigin
        self.body = .text(text)
        self.timestamp = timestamp
    }

    init(id: UUID = UUID(), cortexCycle: CortexCycleMessagePayload, timestamp: Date = Date()) {
        self.id = id
        self.role = .organActivity
        self.signalOrigin = .observability
        self.body = .cortexCycle(cortexCycle)
        self.timestamp = timestamp
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(UUID.self, forKey: .id)
        role = try container.decode(ChatRole.self, forKey: .role)
        signalOrigin = try container.decodeIfPresent(ChatMessageSignalOrigin.self, forKey: .signalOrigin) ?? .local
        body = try container.decode(ChatMessageBody.self, forKey: .body)
        timestamp = try container.decode(Date.self, forKey: .timestamp)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(role, forKey: .role)
        try container.encode(signalOrigin, forKey: .signalOrigin)
        try container.encode(body, forKey: .body)
        try container.encode(timestamp, forKey: .timestamp)
    }

    var text: String {
        switch body {
        case let .text(text):
            return text
        case let .cortexCycle(payload):
            let awakeSuffix: String
            if let awakeSequence = payload.awakeSequence {
                awakeSuffix = ", awake \(awakeSequence)"
            } else {
                awakeSuffix = ""
            }
            return "[cortex cycle] \(payload.cycleID)\(awakeSuffix), \(payload.organActivityMessages.count) organ activity message(s)"
        }
    }
}
