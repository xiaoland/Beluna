import Foundation

let appleEndpointID = "ep:apple-universal:chat"
let appleAffordanceKey = "chat.reply.emit"
let appleCapabilityHandle = "cap.apple.universal.chat"

enum EndpointResultOutcome: Codable, Equatable {
    case applied(actualCostMicro: Int64, referenceID: String)
    case rejected(reasonCode: String, referenceID: String)
    case deferred(reasonCode: String)

    enum CodingKeys: String, CodingKey {
        case type
        case actualCostMicro = "actual_cost_micro"
        case reasonCode = "reason_code"
        case referenceID = "reference_id"
    }

    enum Kind: String, Codable {
        case applied
        case rejected
        case deferred
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(Kind.self, forKey: .type)

        switch kind {
        case .applied:
            self = .applied(
                actualCostMicro: try container.decode(Int64.self, forKey: .actualCostMicro),
                referenceID: try container.decode(String.self, forKey: .referenceID)
            )
        case .rejected:
            self = .rejected(
                reasonCode: try container.decode(String.self, forKey: .reasonCode),
                referenceID: try container.decode(String.self, forKey: .referenceID)
            )
        case .deferred:
            self = .deferred(
                reasonCode: try container.decode(String.self, forKey: .reasonCode)
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        switch self {
        case .applied(let actualCostMicro, let referenceID):
            try container.encode(Kind.applied, forKey: .type)
            try container.encode(actualCostMicro, forKey: .actualCostMicro)
            try container.encode(referenceID, forKey: .referenceID)
        case .rejected(let reasonCode, let referenceID):
            try container.encode(Kind.rejected, forKey: .type)
            try container.encode(reasonCode, forKey: .reasonCode)
            try container.encode(referenceID, forKey: .referenceID)
        case .deferred(let reasonCode):
            try container.encode(Kind.deferred, forKey: .type)
            try container.encode(reasonCode, forKey: .reasonCode)
        }
    }
}

struct RouteDescriptor: Codable, Equatable {
    let affordanceKey: String
    let capabilityHandle: String

    enum CodingKeys: String, CodingKey {
        case affordanceKey = "affordance_key"
        case capabilityHandle = "capability_handle"
    }
}

struct CostVectorWire: Codable, Equatable {
    let survivalMicro: Int64
    let timeMS: UInt64
    let ioUnits: UInt64
    let tokenUnits: UInt64

    enum CodingKeys: String, CodingKey {
        case survivalMicro = "survival_micro"
        case timeMS = "time_ms"
        case ioUnits = "io_units"
        case tokenUnits = "token_units"
    }
}

struct EndpointDescriptorWire: Codable, Equatable {
    let route: RouteDescriptor
    let payloadSchema: JSONValue
    let maxPayloadBytes: Int
    let defaultCost: CostVectorWire
    let metadata: [String: String]

    enum CodingKeys: String, CodingKey {
        case route
        case payloadSchema = "payload_schema"
        case maxPayloadBytes = "max_payload_bytes"
        case defaultCost = "default_cost"
        case metadata
    }
}

struct EndpointRegisterWire: Codable {
    let type = "body_endpoint_register"
    let endpointID: String
    let descriptor: EndpointDescriptorWire

    enum CodingKeys: String, CodingKey {
        case type
        case endpointID = "endpoint_id"
        case descriptor
    }
}

struct EndpointResultWire: Codable {
    let type = "body_endpoint_result"
    let requestID: String
    let outcome: EndpointResultOutcome

    enum CodingKeys: String, CodingKey {
        case type
        case requestID = "request_id"
        case outcome
    }
}

struct SenseWire: Codable {
    let type = "sense"
    let senseID: String
    let source: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case type
        case senseID = "sense_id"
        case source
        case payload
    }
}

struct AdmittedActionWire: Decodable, Equatable {
    let actionID: String
    let affordanceKey: String
    let capabilityHandle: String
    let normalizedPayload: JSONValue
    let reservedCost: CostVectorWire

    enum CodingKeys: String, CodingKey {
        case actionID = "action_id"
        case affordanceKey = "affordance_key"
        case capabilityHandle = "capability_handle"
        case normalizedPayload = "normalized_payload"
        case reservedCost = "reserved_cost"
    }
}

enum ServerWireMessage: Decodable, Equatable {
    case endpointInvoke(requestID: String, action: AdmittedActionWire)

    enum CodingKeys: String, CodingKey {
        case type
        case requestID = "request_id"
        case action
    }

    enum Kind: String, Decodable {
        case endpointInvoke = "body_endpoint_invoke"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(Kind.self, forKey: .type)

        switch kind {
        case .endpointInvoke:
            self = .endpointInvoke(
                requestID: try container.decode(String.self, forKey: .requestID),
                action: try container.decode(AdmittedActionWire.self, forKey: .action)
            )
        }
    }
}

struct ChatReplyInvokePayload: Decodable {
    let conversationID: String
    let response: ResponsePayload

    enum CodingKeys: String, CodingKey {
        case conversationID = "conversation_id"
        case response
    }
}

struct ResponsePayload: Decodable {
    let object: String
    let id: String?
    let output: [ResponseOutputItem]
}

struct ResponseOutputItem: Decodable {
    let type: String
    let role: String?
    let content: [ResponseContentItem]?
}

struct ResponseContentItem: Decodable {
    let type: String
    let text: String?
}

func makeAppleEndpointRegisterEnvelope() -> EndpointRegisterWire {
    EndpointRegisterWire(
        endpointID: appleEndpointID,
        descriptor: EndpointDescriptorWire(
            route: RouteDescriptor(
                affordanceKey: appleAffordanceKey,
                capabilityHandle: appleCapabilityHandle
            ),
            payloadSchema: .object([
                "type": .string("object"),
                "required": .array([.string("conversation_id"), .string("response")])
            ]),
            maxPayloadBytes: 32_768,
            defaultCost: CostVectorWire(
                survivalMicro: 120,
                timeMS: 100,
                ioUnits: 1,
                tokenUnits: 64
            ),
            metadata: ["app": "apple-universal"]
        )
    )
}

func makeUserSenseEnvelope(conversationID: String, text: String) -> SenseWire {
    SenseWire(
        senseID: "sense:apple:\(UUID().uuidString.lowercased())",
        source: "apple.universal.chat",
        payload: .object([
            "conversation_id": .string(conversationID),
            "input": .array([
                .object([
                    "type": .string("message"),
                    "role": .string("user"),
                    "content": .array([
                        .object([
                            "type": .string("input_text"),
                            "text": .string(text)
                        ])
                    ])
                ])
            ])
        ])
    )
}

func extractAssistantTexts(from normalizedPayload: JSONValue) throws -> [String] {
    let payloadData = try JSONEncoder().encode(normalizedPayload)
    let payload = try JSONDecoder().decode(ChatReplyInvokePayload.self, from: payloadData)

    guard payload.response.object == "response" else {
        return []
    }

    var result: [String] = []
    for item in payload.response.output {
        guard item.type == "message" else {
            continue
        }

        for content in item.content ?? [] {
            if content.type == "output_text", let text = content.text, !text.isEmpty {
                result.append(text)
            }
        }
    }

    return result
}

func encodeLine<T: Encodable>(_ value: T) throws -> Data {
    let data = try JSONEncoder().encode(value)
    return data + Data([0x0A])
}

func decodeServerMessage(from line: Data) throws -> ServerWireMessage {
    try JSONDecoder().decode(ServerWireMessage.self, from: line)
}
