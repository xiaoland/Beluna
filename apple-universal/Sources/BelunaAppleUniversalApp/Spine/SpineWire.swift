import Foundation

let appleEndpointID = "macos-app.01"
let appleCapabilityID = "present.message"

struct RouteDescriptor: Codable, Equatable {
    let endpointID: String
    let capabilityID: String

    enum CodingKeys: String, CodingKey {
        case endpointID = "endpoint_id"
        case capabilityID = "capability_id"
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
    let neuralSignalID: String
    let capabilityInstanceID: String
    let endpointID: String
    let capabilityID: String
    let normalizedPayload: JSONValue
    let reservedCost: CostVectorWire

    enum CodingKeys: String, CodingKey {
        case neuralSignalID = "neural_signal_id"
        case capabilityInstanceID = "capability_instance_id"
        case endpointID = "endpoint_id"
        case capabilityID = "capability_id"
        case normalizedPayload = "normalized_payload"
        case reservedCost = "reserved_cost"
    }
}

enum ServerWireMessage: Decodable, Equatable {
    case act(action: AdmittedActionWire)

    enum CodingKeys: String, CodingKey {
        case type
        case action
    }

    enum Kind: String, Decodable {
        case act
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try container.decode(Kind.self, forKey: .type)

        switch kind {
        case .act:
            self = .act(action: try container.decode(AdmittedActionWire.self, forKey: .action))
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
                endpointID: appleEndpointID,
                capabilityID: appleCapabilityID
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

func makeActResultSenseEnvelope(
    action: AdmittedActionWire,
    status: String,
    referenceID: String,
    reasonCode: String? = nil
) -> SenseWire {
    var payload: [String: JSONValue] = [
        "kind": .string("present_message_result"),
        "status": .string(status),
        "neural_signal_id": .string(action.neuralSignalID),
        "capability_instance_id": .string(action.capabilityInstanceID),
        "endpoint_id": .string(action.endpointID),
        "capability_id": .string(action.capabilityID),
        "reference_id": .string(referenceID)
    ]
    if let reasonCode {
        payload["reason_code"] = .string(reasonCode)
    }

    return SenseWire(
        senseID: "sense:apple:\(UUID().uuidString.lowercased())",
        source: "apple.universal.chat",
        payload: .object(payload)
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
